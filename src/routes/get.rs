use crate::{AppState, storage::StorageOperations};
use axum::{
    extract::{Path, State},
    http::{HeaderValue, Response, StatusCode, header},
    response::IntoResponse,
};
use mime_guess::{MimeGuess, mime};
use std::path::PathBuf;

pub async fn get_file_root_handler(State(state): State<AppState>) -> impl IntoResponse {
    serve_file(PathBuf::from("index.html"), state).await
}

pub async fn get_file_handler(
    Path(mut path): Path<PathBuf>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if path.to_string_lossy().ends_with('/') {
        path.push("index.html");
    }
    serve_file(path, state).await
}

async fn serve_file(path: PathBuf, state: AppState) -> impl IntoResponse {
    let Some(file_bytes) = state.storage.read(&path).await.unwrap() else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let content_type = MimeGuess::from_path(&path)
        .first_raw()
        .map(HeaderValue::from_static)
        .unwrap_or_else(|| HeaderValue::from_str(mime::APPLICATION_OCTET_STREAM.as_ref()).unwrap());

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CACHE_CONTROL,
            HeaderValue::from_str(&format!(
                "public, max-age={}, immutable",
                state.file_cache_duration.as_secs()
            ))
            .unwrap(),
        )
        .body(axum::body::Body::from(file_bytes))
        .unwrap()
        .into_response()
}
