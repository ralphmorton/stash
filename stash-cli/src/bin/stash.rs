use clap::Parser;
use stash_client::{Cli, Cmd, Config, exec, keygen};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.cmd {
        Cmd::Keygen => keygen().await?,
        cmd => {
            let config = Config::build()?;
            exec(config.sk, config.server, cmd).await?;
        }
    }

    Ok(())
}
