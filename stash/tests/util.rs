use std::path::PathBuf;

use iroh::{Endpoint, SecretKey, Watcher, protocol::Router};
use sqlx::SqlitePool;
use stash::{Client, Server};

pub struct TestDb(String, SqlitePool);

impl Drop for TestDb {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}

impl TestDb {
    pub async fn new() -> Self {
        let db = format!("test-{}.db", uuid::Uuid::new_v4());

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

        TestDb(db, pool)
    }

    pub fn pool<'a>(&'a self) -> &'a SqlitePool {
        &self.1
    }
}

#[allow(dead_code)]
pub struct ClientServer {
    pub client: Client,
    pub client_sk: SecretKey,
    pub server: Router,
    pub server_sk: SecretKey,
}

impl ClientServer {
    pub async fn new(db: sqlx::SqlitePool, root: PathBuf) -> Self {
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
                Server::new(vec![client_sk.public()], root, db).unwrap(),
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
            client,
            client_sk,
            server,
            server_sk,
        }
    }
}
