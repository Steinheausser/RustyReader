use axum::{extract::{Path, State}, response::IntoResponse, routing::get, Router};
use crate::app_state::AppState;
use crate::error::AppError;

pub fn asset_router() -> Router<AppState> {
    Router::new().route("/resource/:book_id/*path", get(serve_book_resource))
}

async fn serve_book_resource(
    State(state): State<AppState>,
    Path((book_id, path)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let parser = state.parsers.get(&book_id).ok_or(AppError::NotFound)?;
    let bytes = parser.extract_resource(&path)?;
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    
    Ok((
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
        bytes,
    ).into_response())
}
