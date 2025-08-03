mod config;

use config::Config;
use iroh::{Endpoint, NodeId, protocol::Router};
use stash::Server;
use tokio::signal::unix::{SignalKind, signal};

const GATEKEEPER_ROLE: &'static str = "stash";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap();
    let config = Config::build();

    let gk = gatekeeper::Arbiter::new(config.gatekeeper_db_path, true).await?;
    let gk_server = gatekeeper::Server::new(gk.clone());
    let stash_server = Server::new(Auth { gk }, config.root).await?;

    let endpoint = Endpoint::builder()
        .discovery_n0()
        .secret_key(config.secret_key)
        .bind()
        .await?;

    let router = Router::builder(endpoint)
        .accept(gatekeeper::ALPN, gk_server)
        .accept(stash::ALPN, stash_server)
        .spawn();

    let mut sigterm = signal(SignalKind::terminate())?;
    sigterm.recv().await;

    router.shutdown().await?;

    Ok(())
}

struct Auth {
    gk: gatekeeper::Arbiter,
}

impl stash::NodeAuth for Auth {
    async fn allow(&self, node: NodeId) -> bool {
        match self.gk.node_roles(&format!("{node}")).await {
            Err(e) => {
                tracing::error!(err = ?e, "node_auth_failed");
                false
            }
            Ok(roles) => roles.iter().any(|r| r == GATEKEEPER_ROLE),
        }
    }
}
