use envconfig::Envconfig;
use iroh::{NodeId, SecretKey};

#[derive(Clone, Debug, Envconfig)]
pub struct Config {
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
