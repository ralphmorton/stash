use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(about = "File server")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate a new keypair
    Keygen,
    /// Add a new client node (requires admin)
    AddClient {
        /// Node public key
        node: String,
    },
    /// Remove a client node (requires admin)
    RemoveClient {
        /// Node public key
        node: String,
    },
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
    },
    /// Download a file
    Download {
        /// Local file path
        path: PathBuf,
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
