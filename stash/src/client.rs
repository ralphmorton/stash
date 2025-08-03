use bincode::Decode;
use iroh::{Endpoint, NodeAddr, NodeId};

use crate::{ALPN, Blob, Cmd, Error, File, FileDescription, Response, SHA256, Tag, common::Either};

const CHUNK_SIZE: usize = 1_000_000;

#[derive(Clone)]
pub struct Client {
    endpoint: Endpoint,
    server: Either<NodeAddr, NodeId>,
    bincode_config: bincode::config::Configuration,
}

impl Client {
    pub fn new(endpoint: Endpoint, server: NodeId) -> Self {
        Self {
            endpoint,
            server: Either::Right(server),
            bincode_config: bincode::config::standard(),
        }
    }

    pub fn with_addr(endpoint: Endpoint, server: NodeAddr) -> Self {
        Self {
            endpoint,
            server: Either::Left(server),
            bincode_config: bincode::config::standard(),
        }
    }

    pub async fn tags(&self) -> Result<Response<Vec<String>>, Error> {
        self.send(Cmd::Tags).await
    }

    pub async fn create_blob(&self) -> Result<Response<Blob>, Error> {
        self.send(Cmd::CreateBlob).await
    }

    pub async fn describe_blob(&self, name: String) -> Result<Response<Blob>, Error> {
        self.send(Cmd::DescribeBlob { name }).await
    }

    pub async fn append_blob(&self, name: String, data: Vec<u8>) -> Result<Response<Blob>, Error> {
        self.send(Cmd::AppendBlob { name, data }).await
    }

    pub async fn commit_blob(
        &self,
        name: String,
        file_name: String,
        tags: Vec<Tag>,
        replace: bool,
    ) -> Result<Response<File>, Error> {
        let tags = tags.into_iter().map(Into::into).collect();

        self.send(Cmd::CommitBlob {
            name,
            file_name,
            tags,
            replace,
        })
        .await
    }

    pub async fn gc_blobs(&self) -> Result<Response<String>, Error> {
        self.send(Cmd::GcBlobs).await
    }

    pub async fn list(
        &self,
        tag: Tag,
        prefix: Option<String>,
    ) -> Result<Response<Vec<File>>, Error> {
        self.send(Cmd::List {
            tag: tag.into(),
            prefix,
        })
        .await
    }

    pub async fn search(&self, tag: Tag, term: String) -> Result<Response<Vec<File>>, Error> {
        self.send(Cmd::Search {
            tag: tag.into(),
            term,
        })
        .await
    }

    pub async fn describe(&self, name: String) -> Result<Response<FileDescription>, Error> {
        self.send(Cmd::Describe { name }).await
    }

    pub async fn delete(&self, name: String) -> Result<Response<String>, Error> {
        self.send(Cmd::Delete { name }).await
    }

    pub async fn download(
        &self,
        hash: SHA256,
        start: u64,
        len: u64,
    ) -> Result<Response<Vec<u8>>, Error> {
        self.send(Cmd::Download { hash, start, len }).await
    }

    async fn send<R: Decode<()>>(&self, cmd: Cmd) -> Result<R, Error> {
        let json = bincode::encode_to_vec(&cmd, self.bincode_config)?;
        let conn = match &self.server {
            Either::Left(node_id) => self.endpoint.connect(node_id.clone(), ALPN).await?,
            Either::Right(node_addr) => self.endpoint.connect(node_addr.clone(), ALPN).await?,
        };

        let (mut tx, mut rx) = conn.open_bi().await?;
        tx.write_all(&json).await?;
        tx.finish()?;

        let mut data = vec![];
        while let Some(chunk) = rx.read_chunk(CHUNK_SIZE, true).await? {
            let mut bytes = chunk.bytes.to_vec();
            data.append(&mut bytes);
        }

        conn.close(0u32.into(), b"bye");

        let rsp = bincode::decode_from_slice(&data, self.bincode_config)?.0;
        Ok(rsp)
    }
}
