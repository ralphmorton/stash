mod cli;
mod config;

use std::{fmt::Write, os::unix::fs::MetadataExt, path::PathBuf, str::FromStr};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use iroh::{Endpoint, NodeId, SecretKey};
use stash::{Client, File, Tag};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub use cli::{Cli, Cmd};
pub use config::Config;

const CHUNK_SIZE: usize = 5_000_000;

pub async fn exec(sk: SecretKey, server: NodeId, cmd: Cmd) -> anyhow::Result<()> {
    let endpoint = Endpoint::builder()
        .discovery_n0()
        .secret_key(sk)
        .bind()
        .await?;

    let client = Client::new(endpoint, server);

    match cmd {
        Cmd::Keygen => keygen().await,
        Cmd::Tags => tags(client).await,
        Cmd::Upload {
            path,
            name,
            tags,
            replace,
        } => upload(client, path, name, tags, replace).await,
        Cmd::Download { path, name } => download(client, path, name).await,
        Cmd::Read { name } => read(client, name).await,
        Cmd::Delete { name } => delete(client, name).await,
        Cmd::GcBlobs => gc_blobs(client).await,
        Cmd::List { tag, prefix } => list(client, tag, prefix).await,
        Cmd::Search { tag, term } => search(client, tag, term).await,
    }
}

pub async fn keygen() -> anyhow::Result<()> {
    let mut rng = rand::thread_rng();
    let sk = SecretKey::generate(&mut rng);

    let secret = data_encoding::HEXLOWER.encode(&sk.to_bytes());
    let public = format!("{}", sk.public());

    println!("Secret: {secret}\nPublic: {public}");
    Ok(())
}

async fn tags(client: Client) -> anyhow::Result<()> {
    let tags = client.tags().await?.res()?;

    println!("{}", tags.join("\n"));
    Ok(())
}

async fn upload(
    client: Client,
    path: PathBuf,
    name: String,
    tags: Vec<String>,
    replace: bool,
) -> anyhow::Result<()> {
    let tags = tags
        .iter()
        .map(|t| parse_tag(t))
        .collect::<Result<Vec<Tag>, anyhow::Error>>()?;

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

    progress.finish();

    let file = client
        .commit_blob(blob.name, name, tags, replace)
        .await?
        .res()?;

    println!("{}", display_file(&file));
    Ok(())
}

async fn download(client: Client, path: PathBuf, name: String) -> anyhow::Result<()> {
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

async fn read(client: Client, name: String) -> anyhow::Result<()> {
    let remote_file = client.describe(name).await?.res()?;

    let mut stdout = tokio::io::stdout();

    let mut cursor = 0;
    while cursor < remote_file.size {
        let len = std::cmp::min(CHUNK_SIZE as u64, remote_file.size - cursor);
        let chunk = client
            .download(remote_file.hash.clone(), cursor, len)
            .await?
            .res()?;

        stdout.write(&chunk).await?;

        cursor += len;
    }

    stdout.flush().await?;

    Ok(())
}

async fn delete(client: Client, name: String) -> anyhow::Result<()> {
    let rsp = client.delete(name).await?.res()?;

    println!("{rsp}");
    Ok(())
}

async fn gc_blobs(client: Client) -> anyhow::Result<()> {
    let rsp = client.gc_blobs().await?.res()?;

    println!("{rsp}");
    Ok(())
}

async fn list(client: Client, tag: String, prefix: Option<String>) -> anyhow::Result<()> {
    let tag = parse_tag(&tag)?;

    let files = client.list(tag, prefix).await?.res()?;
    for file in files.iter() {
        println!("{}", display_file(file));
    }

    Ok(())
}

async fn search(client: Client, tag: String, term: String) -> anyhow::Result<()> {
    let tag = parse_tag(&tag)?;

    let files = client.search(tag, term).await?.res()?;
    for file in files.iter() {
        println!("{}", display_file(file));
    }

    Ok(())
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
