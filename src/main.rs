mod routes;
mod storage;

use anyhow::Result;
use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, header},
    middleware::{self as axum_middleware, Next},
    routing::get,
};
use clap::Parser;
use clap_duration::duration_range_value_parse;
use dotenvy::dotenv;
use duration_human::{DurationHuman, DurationHumanValidator};
use routes::{get_file_handler, get_file_root_handler};
use std::{net::SocketAddr, time::Duration};
use storage::StorageBackend;
use tokio::{net::TcpListener, signal};
use tower_http::{
    catch_panic::CatchPanicLayer,
    normalize_path::NormalizePathLayer,
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version)]
struct Arguments {
    /// Internet socket address that the server should be ran on.
    #[arg(
        long = "address",
        env = "HERMES_ADDRESS",
        default_value = "127.0.0.1:8080"
    )]
    address: SocketAddr,

    /// The storage backend to serve files from.
    ///
    /// Available options depend on what was enabled at compile time, a full list of backends is below.
    ///
    /// Backends: `fs://<path>`, `s3://bucket`, `sshfs://<mountpoint>`
    #[arg(long = "storage", env = "HERMES_STORAGE_BACKEND")]
    storage: StorageBackend,

    /// The duration of time to cache files for. Files will not be revalidated by the client during this time.
    #[clap(long = "file-cache-duration", env = "HERMES_FILE_CACHE_DURATION", default_value = "1min", value_parser = duration_range_value_parse!(min: 1min, max: 100years))]
    file_cache_duration: DurationHuman,
}

#[derive(Clone)]
struct AppState {
    storage: StorageBackend,
    file_cache_duration: Duration,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info")))
        .init();
    let args = Arguments::parse();

    let tcp_listener = TcpListener::bind(args.address).await?;
    let router = Router::new()
        .route("/", get(get_file_root_handler))
        .route("/{*path}", get(get_file_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default())
                .on_request(DefaultOnRequest::default())
                .on_response(DefaultOnResponse::default().level(Level::INFO))
                .on_failure(DefaultOnFailure::default()),
        )
        .layer(NormalizePathLayer::trim_trailing_slash())
        .layer(CatchPanicLayer::new())
        .layer(axum_middleware::from_fn(
            async |req: Request, next: Next| {
                let mut res = next.run(req).await;
                let res_headers = res.headers_mut();
                res_headers.insert(
                    header::SERVER,
                    HeaderValue::from_static(env!("CARGO_PKG_NAME")),
                );
                res_headers.insert("X-Robots-Tag", HeaderValue::from_static("none"));
                res
            },
        ))
        .with_state(AppState {
            storage: args.storage,
            file_cache_duration: Duration::from(&args.file_cache_duration),
        });

    info!(
        "Internal server started - listening on: http://{}",
        args.address,
    );

    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

// https://github.com/tokio-rs/axum/blob/15917c6dbcb4a48707a20e9cfd021992a279a662/examples/graceful-shutdown/src/main.rs#L55
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
