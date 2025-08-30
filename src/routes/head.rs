use crate::{AppState, storage::StorageOperations};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, Response, StatusCode, header},
    response::IntoResponse,
};
use mime_guess::{MimeGuess, mime};
use std::path::PathBuf;

pub async fn head_file_root_handler(State(state): State<AppState>) -> impl IntoResponse {
    file_metadata(PathBuf::from("index.html"), state).await
}

pub async fn head_file_handler(
    Path(mut path): Path<PathBuf>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if path.to_string_lossy().ends_with('/') {
        path.push("index.html");
    }
    file_metadata(path, state).await
}

async fn file_metadata(path: PathBuf, state: AppState) -> impl IntoResponse {
    let Some(metadata) = state.storage.metadata(&path).await.unwrap() else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let content_type = MimeGuess::from_path(&path)
        .first_raw()
        .map(HeaderValue::from_static)
        .unwrap_or_else(|| HeaderValue::from_str(mime::APPLICATION_OCTET_STREAM.as_ref()).unwrap());
    let mut response_builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type);

    if let Some(cache_duration) = state.file_cache_duration {
        response_builder = response_builder.header(
            header::CACHE_CONTROL,
            HeaderValue::from_str(&format!(
                "public, max-age={}, immutable",
                cache_duration.as_secs()
            ))
            .unwrap(),
        );
    }

    response_builder
        .header(header::CONTENT_LENGTH, metadata.file_size)
        .body(Body::empty())
        .unwrap()
        .into_response()
}
