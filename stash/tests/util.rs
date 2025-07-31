use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use iroh::{Endpoint, NodeId, SecretKey, Watcher, protocol::Router};
use sqlx::SqlitePool;
use stash::{Client, NodeAuth, Server};
use uuid::Uuid;

pub struct TestInfra {
    pub root: PathBuf,
    pub pool: SqlitePool,
}

impl Drop for TestInfra {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

#[allow(dead_code)]
impl TestInfra {
    pub async fn new() -> Self {
        let root = PathBuf::from(format!("test-infra-{}", Uuid::new_v4().to_string()));
        std::fs::create_dir(&root).unwrap();

        let db = format!("{}/test.db", root.display());

        std::process::Command::new("diesel")
            .arg("migration")
            .arg("run")
            .arg("--migration-dir")
            .arg("./migrations")
            .arg("--database-url")
            .arg(&db)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        std::thread::sleep(std::time::Duration::from_secs(2));

        let pool = SqlitePool::connect(&db).await.unwrap();

        TestInfra { root, pool }
    }

    pub async fn blobs(&self) -> Vec<String> {
        let mut blobs_dir = tokio::fs::read_dir(self.root.join("blobs")).await.unwrap();

        let mut blobs = vec![];
        while let Some(entry) = blobs_dir.next_entry().await.unwrap() {
            if entry.file_type().await.unwrap().is_file() {
                blobs.push(entry.file_name().into_string().unwrap());
            }
        }

        blobs
    }

    pub async fn files(&self) -> Vec<String> {
        let mut files_dir = tokio::fs::read_dir(self.root.join("files")).await.unwrap();

        let mut files = vec![];
        while let Some(entry) = files_dir.next_entry().await.unwrap() {
            if entry.file_type().await.unwrap().is_file() {
                files.push(entry.file_name().into_string().unwrap());
            }
        }

        files
    }
}

struct TestAuth {
    admin: NodeId,
    allow: Arc<RwLock<Vec<NodeId>>>,
}

impl NodeAuth for TestAuth {
    async fn allow(&self, node: NodeId) -> bool {
        if node == self.admin {
            return true;
        }

        self.allow.read().unwrap().iter().any(|n| n == &node)
    }

    async fn add(&self, caller: NodeId, node: NodeId) -> bool {
        if caller != self.admin {
            return false;
        }

        self.allow.write().unwrap().push(node);
        true
    }

    async fn remove(&self, caller: NodeId, node: NodeId) -> bool {
        if caller != self.admin {
            return false;
        }

        let mut nodes = self.allow.write().unwrap();
        let allowed = nodes.clone().into_iter().filter(|n| n != &node).collect();
        *nodes = allowed;
        true
    }
}

#[allow(dead_code)]
pub struct ClientServer {
    pub infra: TestInfra,
    pub client: Client,
    pub client_sk: SecretKey,
    pub server: Router,
    pub server_sk: SecretKey,
}

impl ClientServer {
    pub async fn new(infra: TestInfra) -> Self {
        let mut rng = rand::thread_rng();
        let server_sk = SecretKey::generate(&mut rng);
        let client_sk = SecretKey::generate(&mut rng);

        let server_endpoint = Endpoint::builder()
            .discovery_n0()
            .secret_key(server_sk.clone())
            .bind()
            .await
            .unwrap();

        let server = Router::builder(server_endpoint)
            .accept(
                stash::ALPN,
                Server::new(
                    TestAuth {
                        admin: client_sk.public(),
                        allow: Arc::new(RwLock::new(vec![])),
                    },
                    infra.root.clone(),
                    infra.pool.clone(),
                )
                .unwrap(),
            )
            .spawn();

        let server_addr = server.endpoint().node_addr().initialized().await.unwrap();

        let client_endpoint = Endpoint::builder()
            .discovery_n0()
            .secret_key(client_sk.clone())
            .bind()
            .await
            .unwrap();

        let client = stash::Client::with_addr(client_endpoint, server_addr);

        Self {
            infra,
            client,
            client_sk,
            server,
            server_sk,
        }
    }
}
