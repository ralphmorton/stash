mod auth;
mod config;

use auth::Auth;
use config::Config;
use iroh::{Endpoint, protocol::Router};
use sqlx::SqlitePool;
use stash::Server;
use tokio::signal::unix::{SignalKind, signal};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap();
    let config = Config::build();

    let daemon_db = SqlitePool::connect(&config.db).await?;
    let auth = Auth::new(daemon_db, config.admin).await?;

    let server_db_path = config.root.join("server.db");
    let server_db = SqlitePool::connect(server_db_path.to_str().unwrap()).await?;
    let server = Server::new(auth, config.root, server_db)?;

    let endpoint = Endpoint::builder()
        .discovery_n0()
        .secret_key(config.secret_key)
        .bind()
        .await?;

    let router = Router::builder(endpoint)
        .accept(stash::ALPN, server)
        .spawn();

    let mut sigterm = signal(SignalKind::terminate())?;
    sigterm.recv().await;

    router.shutdown().await?;

    Ok(())
}
