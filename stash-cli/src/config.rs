use envconfig::Envconfig;
use iroh::{NodeId, SecretKey};

#[derive(Clone, Debug, Envconfig)]
pub struct Config {
    #[envconfig(from = "STASH_SECRET_KEY")]
    pub sk: SecretKey,

    #[envconfig(from = "STASH_SERVER")]
    pub server: NodeId,
}

impl Config {
    pub fn build() -> anyhow::Result<Self> {
        let config = Self::init_from_env()?;
        Ok(config)
    }
}
