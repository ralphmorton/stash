use std::path::PathBuf;

use envconfig::Envconfig;
use iroh::{NodeId, SecretKey};

#[derive(Clone, Debug, Envconfig)]
pub struct Config {
    #[envconfig(from = "DATABASE_URL")]
    pub db: String,

    #[envconfig(from = "STASH_ROOT")]
    pub root: PathBuf,

    #[envconfig(from = "STASH_SECRET_KEY")]
    pub secret_key: SecretKey,

    #[envconfig(from = "STASH_ADMIN")]
    pub admin: NodeId,
}

impl Config {
    pub fn build() -> Self {
        Self::init_from_env().unwrap()
    }
}
