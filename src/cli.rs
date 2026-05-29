use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Fast local e-reader")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Open a specific book
    Open {
        path: Option<PathBuf>,
        #[arg(long)]
        no_browser: bool,
        #[arg(long, default_value_t = 8765)]
        port: u16,
        #[arg(long)]
        background: bool,
    },
    /// Serve a library directory
    Serve {
        dir: PathBuf,
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
}
