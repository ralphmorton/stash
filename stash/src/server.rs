use std::{fmt::Debug, io::SeekFrom, os::unix::fs::MetadataExt, path::PathBuf, str::FromStr};

use iroh::{
    NodeId,
    endpoint::Connection,
    protocol::{AcceptError, ProtocolHandler},
};
use sqlx::SqlitePool;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use uuid::Uuid;

use crate::SHA256;

use super::{Blob, Cmd, Error, File, Response, Tag, db, sha256};

const BLOB_DIR: &'static str = "blobs";
const FILE_DIR: &'static str = "files";

#[derive(Clone)]
pub struct Server {
    allowed_nodes: Vec<NodeId>,
    root: PathBuf,
    db: SqlitePool,
    bincode_config: bincode::config::Configuration,
}

impl Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Server {{ allowed_nodes: {:?}, root: {:?}, db: {:?} }}",
            self.allowed_nodes, self.root, self.db
        )?;

        Ok(())
    }
}

impl Server {
    pub fn new(allowed_nodes: Vec<NodeId>, root: PathBuf, db: SqlitePool) -> Result<Self, Error> {
        let i = Self {
            allowed_nodes,
            root: root.canonicalize()?,
            db,
            bincode_config: bincode::config::standard(),
        };

        Ok(i)
    }

    async fn handle(&self, cmd: Cmd) -> Result<Vec<u8>, Error> {
        tracing::info!(cmd = ?cmd, "handle");

        let json = match cmd {
            Cmd::AllTags => {
                let tags = self.all_tags().await?;
                bincode::encode_to_vec(&tags, self.bincode_config)?
            }
            Cmd::CreateBlob => {
                let blob = self.create_blob().await?;
                bincode::encode_to_vec(&blob, self.bincode_config)?
            }
            Cmd::DescribeBlob { name } => {
                let blob = self.describe_blob(name).await?;
                bincode::encode_to_vec(&blob, self.bincode_config)?
            }
            Cmd::AppendBlob { name, data } => {
                let blob = self.append_blob(name, data).await?;
                bincode::encode_to_vec(&blob, self.bincode_config)?
            }
            Cmd::CommitBlob {
                name,
                file_name,
                tags,
            } => {
                let file = self.commit_blob(name, file_name, tags).await?;
                bincode::encode_to_vec(&file, self.bincode_config)?
            }
            Cmd::List { tag, prefix } => {
                let files = self.list(tag, prefix).await?;
                bincode::encode_to_vec(&files, self.bincode_config)?
            }
            Cmd::Search { tag, term } => {
                let files = self.search(tag, term).await?;
                bincode::encode_to_vec(&files, self.bincode_config)?
            }
            Cmd::Tags { name } => {
                let tags = self.tags(name).await?;
                bincode::encode_to_vec(&tags, self.bincode_config)?
            }
            Cmd::Delete { name } => {
                let rsp = self.delete(name).await?;
                bincode::encode_to_vec(&rsp, self.bincode_config)?
            }
            Cmd::Download { hash, start, len } => {
                let data = self.download(hash, start, len).await?;
                bincode::encode_to_vec(&data, self.bincode_config)?
            }
        };

        Ok(json)
    }

    async fn all_tags(&self) -> Result<Response<Vec<String>>, Error> {
        let tags = db::Tag::all(&self.db)
            .await?
            .into_iter()
            .map(|t| t.name)
            .collect();

        let rsp = Response::Ok(tags);
        Ok(rsp)
    }

    async fn create_blob(&self) -> Result<Response<Blob>, Error> {
        let name = Uuid::new_v4().to_string();
        let path = self.blob_path(&name)?;

        tokio::fs::File::create(&path).await?;

        self.describe_blob(name).await
    }

    async fn describe_blob(&self, name: String) -> Result<Response<Blob>, Error> {
        let path = self.blob_path(&name)?;
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

    async fn append_blob(&self, name: String, data: Vec<u8>) -> Result<Response<Blob>, Error> {
        let path = self.blob_path(&name)?;
        if !std::fs::exists(&path)? {
            return Ok(Response::Err("No such blob".to_string()));
        }

        let mut file = tokio::fs::File::options().append(true).open(&path).await?;
        file.write_all(&data).await?;
        file.flush().await?;

        self.describe_blob(name).await
    }

    async fn commit_blob(
        &self,
        name: String,
        file_name: String,
        tags: Vec<String>,
    ) -> Result<Response<File>, Error> {
        if tags.is_empty() {
            return Ok(Response::Err(format!("At least one tag is required")));
        }

        for tag in tags.iter() {
            if Tag::from_str(&tag).is_err() {
                return Ok(Response::Err(format!("Invalid tag {tag}")));
            }
        }

        if let Some(_) = db::File::by_name(&self.db, &file_name).await? {
            return Ok(Response::Err(format!("File already exists")));
        }

        let blob_path = self.blob_path(&name)?;
        if !std::fs::exists(&blob_path)? {
            return Ok(Response::Err("No such blob".to_string()));
        }

        let meta = tokio::fs::metadata(&blob_path).await?;
        let hash = sha256::digest(&blob_path).await?;
        let file_path = self.file_path(&hash)?;

        let mut transaction = self.db.begin().await?;

        let content = match db::FileContent::by_hash(&mut *transaction, &hash).await? {
            Some(content) => content,
            None => db::FileContent::insert(&mut *transaction, meta.size() as i64, &hash).await?,
        };
        let file = db::File::insert(&mut *transaction, &file_name, content.id).await?;

        for tag in tags.iter() {
            let tag = match db::Tag::by_name(&mut *transaction, tag).await? {
                Some(tag) => tag,
                None => db::Tag::insert(&mut *transaction, tag).await?,
            };

            db::FileTag::insert(&mut *transaction, file.id, tag.id).await?;
        }

        let file = File {
            name: file_name,
            size: meta.size(),
            hash,
            created: file.created.and_utc().timestamp(),
        };

        tokio::fs::rename(&blob_path, &file_path).await?;

        transaction.commit().await?;
        Ok(Response::Ok(file))
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

    async fn tags(&self, name: String) -> Result<Response<Vec<String>>, Error> {
        match db::File::by_name(&self.db, &name).await? {
            None => Ok(Response::Err("No such file".to_string())),
            Some(file) => {
                let tags = db::FileTag::for_file(&self.db, file.id).await?;
                Ok(Response::Ok(tags))
            }
        }
    }

    async fn delete(&self, name: String) -> Result<Response<String>, Error> {
        match db::File::by_name(&self.db, &name).await? {
            None => Ok(Response::Err("No such file".to_string())),
            Some(file) => {
                db::File::delete(&self.db, file.id).await?;
                Ok(Response::ok())
            }
        }
    }

    async fn download(
        &self,
        hash: SHA256,
        start: u64,
        len: u64,
    ) -> Result<Response<Vec<u8>>, Error> {
        let path = self.file_path(&hash)?;
        if !path.exists() {
            return Ok(Response::Err("No such file".to_string()));
        }

        let meta = tokio::fs::metadata(&path).await?;
        if meta.size() < start + len {
            return Ok(Response::Err("Data index out of bounds".to_string()));
        }

        let mut file = tokio::fs::File::open(&path).await?;
        file.seek(SeekFrom::Start(start)).await?;

        let mut data = vec![0; len as usize];
        file.read_exact(&mut data).await?;

        Ok(Response::Ok(data))
    }

    fn blob_path(&self, name: &str) -> Result<PathBuf, Error> {
        let blobs_path = self.root.join(BLOB_DIR);
        if !std::fs::exists(&blobs_path)? {
            std::fs::create_dir(&blobs_path)?;
        }

        Ok(blobs_path.join(name))
    }

    fn file_path(&self, hash: &SHA256) -> Result<PathBuf, Error> {
        let files_path = self.root.join(FILE_DIR);
        if !std::fs::exists(&files_path)? {
            std::fs::create_dir(&files_path)?;
        }

        Ok(files_path.join(hash))
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

        let cmd: Cmd = bincode::decode_from_slice(&data, self.bincode_config)
            .map_err(AcceptError::from_err)?
            .0;

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
