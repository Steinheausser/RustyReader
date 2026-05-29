use fast_ereader::*;

use clap::Parser;
use cli::{Cli, Commands};
use tracing::info;
use std::sync::Arc;
use app_state::AppStateInner;
use parser::Parser as BookParser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("fast-ereader.log")?;
        
    tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(log_file))
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("fast_ereader=info".parse()?))
        .init();

    let cli = Cli::parse();

    let command = cli.command.unwrap_or_else(|| Commands::Open {
        path: None,
        no_browser: true,
        port: 8765,
        background: true,
    });

    match command {
        Commands::Open { path, no_browser, port, background: _ } => {
            info!("Starting server...");
            
            let state = Arc::new(AppStateInner::new());
            
            let mut book_id_to_open = None;
            
            if let Some(p) = path {
                info!("Opening book: {:?}", p);
                // Parse book
                let parser = parser::epub::EpubParser::new(&p)?;
                let book = parser::epub::EpubParser::parse_book(&p)?;
                let book_id = book.id.clone();
                
                // Insert into state
                state.books.insert(book_id.clone(), book);
                state.parsers.insert(book_id.clone(), Arc::new(parser));
                book_id_to_open = Some(book_id);
            }
            
            // Start server
            server::start(state, port, !no_browser, book_id_to_open.as_deref()).await?;
        }
        Commands::Serve { dir, port: _ } => {
            info!("Serving library: {:?}", dir);
            // Start library server mode (Not implemented yet)
        }
    }
    
    Ok(())
}
