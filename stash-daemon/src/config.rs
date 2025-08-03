use std::path::PathBuf;

use envconfig::Envconfig;
use iroh::SecretKey;

#[derive(Clone, Debug, Envconfig)]
pub struct Config {
    #[envconfig(from = "GATEKEEPER_DB_PATH")]
    pub gatekeeper_db_path: PathBuf,

    #[envconfig(from = "STASH_ROOT")]
    pub root: PathBuf,

    #[envconfig(from = "STASH_SECRET_KEY")]
    pub secret_key: SecretKey,
}

impl Config {
    pub fn build() -> Self {
        Self::init_from_env().unwrap()
    }
}
