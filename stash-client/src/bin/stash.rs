use std::{fmt::Write, os::unix::fs::MetadataExt, path::PathBuf, str::FromStr};

use clap::Parser;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use iroh::{Endpoint, NodeId, SecretKey};
use stash::{Client, File, Tag};
use stash_client::{Cli, Command, Config};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const CHUNK_SIZE: usize = 1_000_000;

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
        Command::Tags => tags().await,
        Command::Upload {
            path,
            name,
            tags,
            replace,
        } => upload(path, name, tags, replace).await,
        Command::Download { path, name } => download(path, name).await,
        Command::Read { name } => read(name).await,
        Command::Delete { name } => delete(name).await,
        Command::GcBlobs => gc_blobs().await,
        Command::List { tag, prefix } => list(tag, prefix).await,
        Command::Search { tag, term } => search(tag, term).await,
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

async fn tags() -> anyhow::Result<()> {
    let client = client().await?;
    let tags = client.tags().await?.res()?;

    println!("{}", tags.join("\n"));
    Ok(())
}

async fn upload(
    path: PathBuf,
    name: String,
    tags: Vec<String>,
    replace: bool,
) -> anyhow::Result<()> {
    let tags = tags
        .iter()
        .map(|t| parse_tag(t))
        .collect::<Result<Vec<Tag>, anyhow::Error>>()?;

    let client = client().await?;
    let mut file = tokio::fs::File::open(path).await?;
    let meta = file.metadata().await?;
    let blob = client.create_blob().await?.res()?;

    let mut written = 0;
    let progress = progress_bar(meta.size());

    let mut buf = vec![0; CHUNK_SIZE];
    loop {
        progress.set_position(written as u64);

        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        client
            .append_blob(blob.name.clone(), buf[0..n].to_vec())
            .await?
            .res()?;

        written += n;
    }

    let file = client
        .commit_blob(blob.name, name, tags, replace)
        .await?
        .res()?;
    progress.finish();

    println!("{}", display_file(&file));
    Ok(())
}

async fn download(path: PathBuf, name: String) -> anyhow::Result<()> {
    let client = client().await?;
    let remote_file = client.describe(name).await?.res()?;

    let temp_path = format!("{}.stashdl", path.display());
    let mut local_file = tokio::fs::File::create(&temp_path).await?;

    let progress = progress_bar(remote_file.size);

    let mut cursor = 0;
    while cursor < remote_file.size {
        progress.set_position(cursor);
        let len = std::cmp::min(CHUNK_SIZE as u64, remote_file.size - cursor);
        let chunk = client
            .download(remote_file.hash.clone(), cursor, len)
            .await?
            .res()?;
        local_file.write_all(&chunk).await?;
        cursor += len;
    }

    local_file.flush().await?;
    tokio::fs::rename(temp_path, path).await?;
    progress.finish();

    println!("OK");
    Ok(())
}

async fn read(name: String) -> anyhow::Result<()> {
    let client = client().await?;
    let remote_file = client.describe(name).await?.res()?;

    let mut cursor = 0;
    while cursor < remote_file.size {
        let len = std::cmp::min(CHUNK_SIZE as u64, remote_file.size - cursor);
        let chunk = client
            .download(remote_file.hash.clone(), cursor, len)
            .await?
            .res()?;

        tokio::io::stdout().write(&chunk).await?;

        cursor += len;
    }

    Ok(())
}

async fn delete(name: String) -> anyhow::Result<()> {
    let client = client().await?;
    let rsp = client.delete(name).await?.res()?;

    println!("{rsp}");
    Ok(())
}

async fn gc_blobs() -> anyhow::Result<()> {
    let client = client().await?;
    let rsp = client.gc_blobs().await?.res()?;

    println!("{rsp}");
    Ok(())
}

async fn list(tag: String, prefix: Option<String>) -> anyhow::Result<()> {
    let tag = parse_tag(&tag)?;
    let client = client().await?;

    let files = client.list(tag, prefix).await?.res()?;
    for file in files.iter() {
        println!("{}", display_file(file));
    }

    Ok(())
}

async fn search(tag: String, term: String) -> anyhow::Result<()> {
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

fn progress_bar(total: u64) -> ProgressBar {
    let progress = ProgressBar::new(total);
    progress
        .set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    progress
}
