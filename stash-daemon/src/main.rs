mod config;

use crate::config::Config;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    dotenvy::dotenv().unwrap();
    let config = Config::build();

    println!("{config:?}");
}
