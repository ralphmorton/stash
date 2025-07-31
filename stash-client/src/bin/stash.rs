use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use iroh::{Endpoint, NodeId, SecretKey};
use stash::{Client, File, Tag};
use stash_client::{Cli, Command, Config};
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Keygen => keygen().await,
        Command::AddClient { node } => add_client(node).await,
        Command::RemoveClient { node } => remove_client(node).await,
        Command::ListTags => list_tags().await,
        Command::Upload { path, name, tags } => upload(path, name, tags).await,
        Command::GcBlobs => gc_blobs().await,
        Command::ListFiles { tag, prefix } => list_files(tag, prefix).await,
        Command::SearchFiles { tag, term } => search_files(tag, term).await,
    }
}

async fn keygen() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    let sk = SecretKey::generate(&mut rng);

    let secret = data_encoding::HEXLOWER.encode(&sk.to_bytes());
    let public = format!("{}", sk.public());

    println!("Secret: {secret}\nPublic: {public}");
    Ok(())
}

async fn add_client(node: String) -> anyhow::Result<()> {
    let node = NodeId::from_str(&node)?;
    let client = client().await?;
    let rsp = client.add_client(node).await?.res()?;

    println!("{rsp}");
    Ok(())
}

async fn remove_client(node: String) -> anyhow::Result<()> {
    let node = NodeId::from_str(&node)?;
    let client = client().await?;
    let rsp = client.remove_client(node).await?.res()?;

    println!("{rsp}");
    Ok(())
}

async fn list_tags() -> anyhow::Result<()> {
    let client = client().await?;
    let tags = client.all_tags().await?.res()?;

    println!("{}", tags.join("\n"));
    Ok(())
}

async fn upload(path: PathBuf, name: String, tags: Vec<String>) -> anyhow::Result<()> {
    let tags = tags
        .iter()
        .map(|t| parse_tag(t))
        .collect::<Result<Vec<Tag>, anyhow::Error>>()?;

    let client = client().await?;
    let mut file = tokio::fs::File::open(path).await?;
    let blob = client.create_blob().await?.res()?;

    let mut buf = vec![0; 1_000_000];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        client
            .append_blob(blob.name.clone(), buf[0..n].to_vec())
            .await?
            .res()?;
    }

    let file = client.commit_blob(blob.name, name, tags).await?.res()?;
    println!("{file:?}");
    Ok(())
}

async fn gc_blobs() -> anyhow::Result<()> {
    let client = client().await?;
    let rsp = client.gc_blobs().await?.res()?;

    println!("{rsp}");
    Ok(())
}

async fn list_files(tag: String, prefix: Option<String>) -> anyhow::Result<()> {
    let tag = parse_tag(&tag)?;
    let client = client().await?;

    let files = client.list(tag, prefix).await?.res()?;
    for file in files.iter() {
        println!("{}", display_file(file));
    }

    Ok(())
}

async fn search_files(tag: String, term: String) -> anyhow::Result<()> {
    let tag = parse_tag(&tag)?;
    let client = client().await?;

    let files = client.search(tag, term).await?.res()?;
    for file in files.iter() {
        println!("{}", display_file(file));
    }

    Ok(())
}

async fn client() -> anyhow::Result<Client> {
    let config = Config::build();

    let endpoint = Endpoint::builder()
        .discovery_n0()
        .secret_key(config.secret_key)
        .bind()
        .await?;

    Ok(Client::new(endpoint, config.server))
}

fn parse_tag(tag: &str) -> anyhow::Result<Tag> {
    Tag::from_str(tag).map_err(|_| anyhow::anyhow!("Invalid tag {tag}"))
}

fn display_file(file: &File) -> String {
    format!(
        "{} {} {}\t{}",
        file.created, file.hash, file.size, file.name
    )
}
