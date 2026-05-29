use axum::Router;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::info;
use crate::app_state::AppState;
use crate::routes::{api::api_router, assets::asset_router};

pub async fn start(state: AppState, port: u16, open_browser: bool, book_id: Option<&str>) -> anyhow::Result<()> {
    let app = Router::new()
        .nest("/api", api_router())
        .nest("/assets", asset_router())
        .fallback_service(ServeDir::new("static"))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // If port 0 was requested, listener.local_addr() will have the actual bound port
    let local_addr = listener.local_addr()?;
    info!("Server listening on http://{}", local_addr);
    
    if open_browser {
        let mut url = format!("http://127.0.0.1:{}", local_addr.port());
        if let Some(id) = book_id {
            url.push_str(&format!("?book={}", id));
        }
        
        if let Err(e) = webbrowser::open(&url) {
            tracing::error!("Failed to open web browser: {}", e);
        } else {
            info!("Opened browser at {}", url);
        }
    }
    
    axum::serve(listener, app).await?;
    Ok(())
}
