use std::{os::unix::fs::MetadataExt, path::PathBuf, str::FromStr};

use iroh::{
    NodeId,
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::{Blob, Cmd, Error, File, Response, Tag, db};

const BLOB_DIR: &'static str = "blobs";
const FILE_DIR: &'static str = "files";

#[derive(Clone, Debug)]
pub struct Server {
    allowed_nodes: Vec<NodeId>,
    root: PathBuf,
    db: SqlitePool,
}

impl Server {
    pub fn new(allowed_nodes: Vec<NodeId>, root: PathBuf, db: SqlitePool) -> Result<Self, Error> {
        let i = Self {
            allowed_nodes,
            root: root.canonicalize()?,
            db,
        };

        Ok(i)
    }

    async fn handle(&self, cmd: Cmd) -> Result<Vec<u8>, Error> {
        tracing::info!(cmd = ?cmd, "handle");

        let json = match cmd {
            Cmd::Tags => {
                let tags = self.tags().await?;
                serde_json::to_vec(&tags)?
            }
            Cmd::CreateTag { tag } => {
                let rsp = self.create_tag(tag).await?;
                serde_json::to_vec(&rsp)?
            }
            Cmd::DeleteTag { tag } => {
                let rsp = self.delete_tag(tag).await?;
                serde_json::to_vec(&rsp)?
            }
            Cmd::CreateBlob => {
                let blob = self.create_blob().await?;
                serde_json::to_vec(&blob)?
            }
            Cmd::DescribeBlob { name } => {
                let blob = self.describe_blob(name).await?;
                serde_json::to_vec(&blob)?
            }
            Cmd::List { tag, prefix } => {
                let files = self.list(tag, prefix).await?;
                serde_json::to_vec(&files)?
            }
            Cmd::Search { tag, term } => {
                let files = self.search(tag, term).await?;
                serde_json::to_vec(&files)?
            }
        };

        Ok(json)
    }

    async fn tags(&self) -> Result<Response<Vec<String>>, Error> {
        let tags = db::Tag::all(&self.db)
            .await?
            .into_iter()
            .map(|t| t.name)
            .collect();

        let rsp = Response::Ok(tags);
        Ok(rsp)
    }

    async fn create_tag(&self, tag: String) -> Result<Response<String>, Error> {
        if Tag::from_str(&tag).is_err() {
            return Ok(Response::Err(format!("Invalid tag {tag}")));
        }

        db::Tag::insert(&self.db, &tag).await?;
        Ok(Response::Ok("OK".to_string()))
    }

    async fn delete_tag(&self, tag: String) -> Result<Response<String>, Error> {
        if Tag::from_str(&tag).is_err() {
            return Ok(Response::Err(format!("Invalid tag {tag}")));
        }

        match db::Tag::by_name(&self.db, &tag).await? {
            None => Ok(Response::Err("No such tag".to_string())),
            Some(tag) => {
                db::Tag::delete(&self.db, tag.id).await?;
                Ok(Response::Ok("OK".to_string()))
            }
        }
    }

    async fn create_blob(&self) -> Result<Response<Blob>, Error> {
        let name = Uuid::new_v4().to_string();
        let path = self.blob_path(&name);

        tokio::fs::File::create(&path).await?;

        self.describe_blob(name).await
    }

    async fn describe_blob(&self, name: String) -> Result<Response<Blob>, Error> {
        let path = self.blob_path(&name);
        if !std::fs::exists(&path)? {
            return Ok(Response::Err("No such blob".to_string()));
        }

        let meta = tokio::fs::metadata(&path).await?;

        let blob = Blob {
            name,
            size: meta.size(),
        };

        Ok(Response::Ok(blob))
    }

    async fn list(
        &self,
        tag: String,
        prefix: Option<String>,
    ) -> Result<Response<Vec<File>>, Error> {
        if Tag::from_str(&tag).is_err() {
            return Ok(Response::Err(format!("Invalid tag {tag}")));
        }

        let term = prefix.as_ref().map(|s| s.as_str()).unwrap_or("");
        let term = format!("{term}%");

        let files = db::File::search(&self.db, &tag, &term)
            .await?
            .into_iter()
            .map(From::from)
            .collect();

        let rsp = Response::Ok(files);
        Ok(rsp)
    }

    async fn search(&self, tag: String, term: String) -> Result<Response<Vec<File>>, Error> {
        if Tag::from_str(&tag).is_err() {
            return Ok(Response::Err(format!("Invalid tag {tag}")));
        }

        let term = format!("%{term}%");

        let files = db::File::search(&self.db, &tag, &term)
            .await?
            .into_iter()
            .map(From::from)
            .collect();

        let rsp = Response::Ok(files);
        Ok(rsp)
    }

    fn blob_path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl ProtocolHandler for Server {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let node_id = connection.remote_node_id()?;
        tracing::info!(node_id = ?node_id, "accept");
        if !self.allowed_nodes.iter().any(|pk| pk == &node_id) {
            tracing::warn!(node_id = ?node_id, "unauthorized_client_node");
            return Err(AcceptError::NotAllowed {});
        }

        let (mut tx, mut rx) = connection.accept_bi().await?;

        let mut data = vec![];
        while let Some(chunk) = rx
            .read_chunk(100_000, true)
            .await
            .map_err(AcceptError::from_err)?
        {
            let mut bytes = chunk.bytes.to_vec();
            data.append(&mut bytes);
        }

        let cmd: Cmd = serde_json::from_slice(&data).map_err(AcceptError::from_err)?;
        let rsp = self.handle(cmd.clone()).await;

        if rsp.is_err() {
            tracing::warn!(cmd = ?cmd, rsp = ?rsp, "handle_failed");
        }

        let rsp = rsp.map_err(AcceptError::from_err)?;

        tx.write_all(&rsp).await.map_err(AcceptError::from_err)?;
        tx.finish()?;
        connection.closed().await;

        Ok(())
    }
}
