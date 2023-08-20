#![forbid(unsafe_code)]

pub mod protocols;
pub mod traits;

use axum::handler::HandlerWithoutStateExt;
use axum::Router;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_http::trace::{self};
use tracing::Level;

/// Options for the app.
pub struct AppOptions {
    pub serve_directory: String,
}

/// Create a new app instance with the given options.
pub fn create_app(app_options: AppOptions) -> Router {
    Router::new()
        .nest_service(
            "/",
            ServeDir::new(&app_options.serve_directory)
                .append_index_html_on_directories(true)
                .not_found_service(handle_404_file.into_service()),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}

/// Handle 404 errors by returning a 404 message.
async fn handle_404_file() -> &'static str {
    "The requested resource could not be found."
}
