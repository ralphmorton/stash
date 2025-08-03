use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about = "File server")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    /// Generate a new keypair
    Keygen,
    /// List tags
    Tags,
    /// Upload a file
    Upload {
        /// Local file path
        path: PathBuf,
        /// Remote file name
        name: String,
        /// Tags (comma-separated)
        #[arg(long, use_value_delimiter = true, value_delimiter = ',')]
        tags: Vec<String>,
        /// Replace existing file?
        #[arg(long, default_value_t = false)]
        replace: bool,
    },
    /// Download a file
    Download {
        /// Local file path
        path: PathBuf,
        /// Remote file name
        name: String,
    },
    /// Read a file, printing its contents to stdout
    Read {
        /// Remote file name
        name: String,
    },
    /// Delete a file
    Delete {
        /// Remote file name
        name: String,
    },
    /// GC blob store
    GcBlobs,
    /// List files
    List {
        /// Tag
        tag: String,
        /// Prefix (optional)
        #[arg(long)]
        prefix: Option<String>,
    },
    /// Search files
    Search {
        /// Tag
        tag: String,
        /// Term
        term: String,
    },
}
