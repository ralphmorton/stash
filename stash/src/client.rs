use iroh::{Endpoint, NodeAddr, NodeId};

use crate::{ALPN, Blob, Cmd, Error, File, Response, SHA256, Tag, common::Either};

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

    pub async fn add_client(&self, node: NodeId) -> Result<Response<String>, Error> {
        let mut bytes = vec![];
        self.request(
            Cmd::AddClient {
                node: format!("{node}"),
            },
            |mut chunk| {
                bytes.append(&mut chunk);
                Ok(())
            },
        )
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn remove_client(&self, node: NodeId) -> Result<Response<String>, Error> {
        let mut bytes = vec![];
        self.request(
            Cmd::RemoveClient {
                node: format!("{node}"),
            },
            |mut chunk| {
                bytes.append(&mut chunk);
                Ok(())
            },
        )
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn all_tags(&self) -> Result<Response<Vec<String>>, Error> {
        let mut bytes = vec![];
        self.request(Cmd::AllTags, |mut chunk| {
            bytes.append(&mut chunk);
            Ok(())
        })
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn create_blob(&self) -> Result<Response<Blob>, Error> {
        let mut bytes = vec![];
        self.request(Cmd::CreateBlob, |mut chunk| {
            bytes.append(&mut chunk);
            Ok(())
        })
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn describe_blob(&self, name: String) -> Result<Response<Blob>, Error> {
        let mut bytes = vec![];
        self.request(Cmd::DescribeBlob { name }, |mut chunk| {
            bytes.append(&mut chunk);
            Ok(())
        })
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn append_blob(&self, name: String, data: Vec<u8>) -> Result<Response<Blob>, Error> {
        let mut bytes = vec![];
        self.request(Cmd::AppendBlob { name, data }, |mut chunk| {
            bytes.append(&mut chunk);
            Ok(())
        })
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn commit_blob(
        &self,
        name: String,
        file_name: String,
        tags: Vec<Tag>,
    ) -> Result<Response<File>, Error> {
        let tags = tags.into_iter().map(Into::into).collect();

        let mut bytes = vec![];
        self.request(
            Cmd::CommitBlob {
                name,
                file_name,
                tags,
            },
            |mut chunk| {
                bytes.append(&mut chunk);
                Ok(())
            },
        )
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn list(
        &self,
        tag: Tag,
        prefix: Option<String>,
    ) -> Result<Response<Vec<File>>, Error> {
        let mut bytes = vec![];
        self.request(
            Cmd::List {
                tag: tag.into(),
                prefix,
            },
            |mut chunk| {
                bytes.append(&mut chunk);
                Ok(())
            },
        )
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn search(&self, tag: Tag, term: String) -> Result<Response<Vec<File>>, Error> {
        let mut bytes = vec![];
        self.request(
            Cmd::Search {
                tag: tag.into(),
                term,
            },
            |mut chunk| {
                bytes.append(&mut chunk);
                Ok(())
            },
        )
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn tags(&self, name: String) -> Result<Response<Vec<String>>, Error> {
        let mut bytes = vec![];
        self.request(Cmd::Tags { name }, |mut chunk| {
            bytes.append(&mut chunk);
            Ok(())
        })
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn delete(&self, name: String) -> Result<Response<String>, Error> {
        let mut bytes = vec![];
        self.request(Cmd::Delete { name }, |mut chunk| {
            bytes.append(&mut chunk);
            Ok(())
        })
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    pub async fn download(
        &self,
        hash: SHA256,
        start: u64,
        len: u64,
    ) -> Result<Response<Vec<u8>>, Error> {
        let mut bytes = vec![];
        self.request(Cmd::Download { hash, start, len }, |mut chunk| {
            bytes.append(&mut chunk);
            Ok(())
        })
        .await?;

        let rsp = bincode::decode_from_slice(&bytes, self.bincode_config)?.0;
        Ok(rsp)
    }

    async fn request<F: FnMut(Vec<u8>) -> Result<(), Error>>(
        &self,
        cmd: Cmd,
        f: F,
    ) -> Result<(), Error> {
        let mut f = f;
        let json = bincode::encode_to_vec(&cmd, self.bincode_config)?;
        let conn = match &self.server {
            Either::Left(node_id) => self.endpoint.connect(node_id.clone(), ALPN).await?,
            Either::Right(node_addr) => self.endpoint.connect(node_addr.clone(), ALPN).await?,
        };

        let (mut tx, mut rx) = conn.open_bi().await?;
        tx.write_all(&json).await?;
        tx.finish()?;

        while let Some(chunk) = rx.read_chunk(100_000, true).await? {
            let bytes = chunk.bytes.to_vec();
            f(bytes)?;
        }

        conn.close(0u32.into(), b"bye");

        Ok(())
    }
}
