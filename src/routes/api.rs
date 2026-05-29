use axum::{extract::{Path, State, Query}, Json, routing::{get, post}, Router, body::Bytes};
use serde::Deserialize;
use std::sync::Arc;
use crate::app_state::AppState;
use crate::error::AppError;
use crate::parser::Parser;
use crate::library::LibraryEntry;
use uuid::Uuid;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/library", get(get_library))
        .route("/library/upload", post(upload_book))
        .route("/book/:id", get(get_book))
        .route("/book/:book_id/chapters/:chapter_id/render", get(render_chapter))
}

async fn health_check() -> &'static str {
    "OK"
}

async fn get_library(State(state): State<AppState>) -> Result<Json<Vec<crate::domain::book::Book>>, AppError> {
    let lib = state.library.read();
    let mut books: Vec<_> = lib.entries.values().map(|e| e.book.clone()).collect();
    books.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(Json(books))
}

#[derive(serde::Serialize)]
struct UploadResponse {
    id: String,
}

async fn upload_book(State(state): State<AppState>, body: Bytes) -> Result<Json<UploadResponse>, AppError> {
    let app_dir = crate::library::Library::get_app_dir();
    let temp_id = Uuid::new_v4().to_string();
    let file_path = app_dir.join(format!("{}.epub", temp_id));
    
    std::fs::write(&file_path, &body).map_err(|e| anyhow::anyhow!("Failed to save upload: {}", e))?;
    
    let parser = crate::parser::epub::EpubParser::new(&file_path)?;
    let book = crate::parser::epub::EpubParser::parse_book(&file_path)?;
    
    let book_id = book.id.clone();
    
    let entry = LibraryEntry {
        book: book.clone(),
        file_path,
    };
    
    {
        let mut lib = state.library.write();
        lib.entries.insert(book_id.clone(), entry);
        let _ = lib.save();
    }
    
    state.books.insert(book_id.clone(), book);
    let arc_p: Arc<dyn Parser + Send + Sync> = Arc::new(parser);
    state.parsers.insert(book_id.clone(), arc_p);
    
    Ok(Json(UploadResponse { id: book_id }))
}

async fn get_book(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<crate::domain::book::Book>, AppError> {
    let book = state.books.get(&id).ok_or(AppError::NotFound)?;
    Ok(Json(book.value().clone()))
}

#[derive(Deserialize)]
pub struct RenderQuery {
    pub bionic: Option<u8>,
}

async fn render_chapter(
    State(state): State<AppState>,
    Path((book_id, chapter_id)): Path<(String, String)>,
    Query(query): Query<RenderQuery>,
) -> Result<axum::response::Html<String>, AppError> {
    let bionic_enabled = query.bionic.unwrap_or(0) == 1;
    let cache_key = crate::util::generate_cache_key(&chapter_id, if bionic_enabled { 1 } else { 0 });
    
    if let Some(cached) = state.render_cache.get(&cache_key) {
        return Ok(axum::response::Html(cached));
    }
    
    let parser = if let Some(p) = state.parsers.get(&book_id) {
        p.clone()
    } else {
        let lib = state.library.read();
        if let Some(entry) = lib.entries.get(&book_id) {
            let p = crate::parser::epub::EpubParser::new(&entry.file_path)?;
            let arc_p: Arc<dyn Parser + Send + Sync> = Arc::new(p);
            state.parsers.insert(book_id.clone(), arc_p.clone());
            arc_p
        } else {
            return Err(AppError::NotFound);
        }
    };
    
    let raw_html = parser.extract_chapter_html(&chapter_id)?;
    let normalized = crate::render::html::normalize_html(&raw_html, &book_id);
    
    let final_html = if bionic_enabled {
        let settings = state.settings.read();
        crate::render::bionic::apply_bionic_reading(&normalized, &settings.bionic)
    } else {
        normalized
    };
    
    state.render_cache.insert(cache_key, final_html.clone());
    
    Ok(axum::response::Html(final_html))
}
