use iroh::{Endpoint, NodeAddr, NodeId};

use crate::{ALPN, Cmd, Error, Response, Tag, common::Either};

#[derive(Clone, Debug)]
pub struct Client {
    endpoint: Endpoint,
    server: Either<NodeAddr, NodeId>,
}

impl Client {
    pub fn new(endpoint: Endpoint, server: NodeId) -> Self {
        Self {
            endpoint,
            server: Either::Right(server),
        }
    }

    pub fn with_addr(endpoint: Endpoint, server: NodeAddr) -> Self {
        Self {
            endpoint,
            server: Either::Left(server),
        }
    }

    pub async fn tags(&self) -> Result<Response<Vec<String>>, Error> {
        let mut bytes = vec![];

        self.request(Cmd::Tags, |mut chunk| {
            bytes.append(&mut chunk);
            Ok(())
        })
        .await?;

        let rsp = serde_json::from_slice(&bytes)?;
        Ok(rsp)
    }

    pub async fn create_tag(&self, tag: Tag) -> Result<Response<String>, Error> {
        let mut bytes = vec![];

        self.request(Cmd::CreateTag { tag: tag.into() }, |mut chunk| {
            bytes.append(&mut chunk);
            Ok(())
        })
        .await?;

        let rsp = serde_json::from_slice(&bytes)?;
        Ok(rsp)
    }

    pub async fn delete_tag(&self, tag: Tag) -> Result<Response<String>, Error> {
        let mut bytes = vec![];

        self.request(Cmd::DeleteTag { tag: tag.into() }, |mut chunk| {
            bytes.append(&mut chunk);
            Ok(())
        })
        .await?;

        let rsp = serde_json::from_slice(&bytes)?;
        Ok(rsp)
    }

    async fn request<F: FnMut(Vec<u8>) -> Result<(), Error>>(
        &self,
        cmd: Cmd,
        f: F,
    ) -> Result<(), Error> {
        let mut f = f;
        let json = serde_json::to_vec(&cmd)?;
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
