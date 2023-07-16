#![forbid(unsafe_code)]

mod program_options;
mod remote_settings;

use crate::program_options::{ProgramOptions, Protocols};
use crate::remote_settings::{FromEnv, SshfsOptions};
use axum::{handler::HandlerWithoutStateExt, http::StatusCode, Router};
use dotenv::dotenv;
use futures::stream::StreamExt;
use remote_mount::protocols::{sshfs::Sshfs, ProtocolHandler};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::net::SocketAddr;
use std::process::exit;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_http::trace::{self};
use tracing::{error, info, Level};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let program_options: ProgramOptions = ProgramOptions::from_env_and_args();

    // If we're using a local filesystem, set it up and start the server without a protocol handler.
    if program_options.protocol == Protocols::Local {
        info!("Using local filesystem");
        let app = create_app(&program_options.serve_dir);

        info!("Serving files from {}", &program_options.serve_dir);
        start_app(app, &program_options.socket_addr).await;
        return;
    }

    // Otherwise, we're using a remote filesystem, so set up the protocol handler.
    info!("Using protocol '{:#?}'", program_options.protocol);
    let mut protocol_handler: Box<dyn ProtocolHandler + Send + Sync> =
        match program_options.protocol {
            Protocols::Sshfs => {
                let sshfs_options = match SshfsOptions::from_env() {
                    Ok(options) => options,
                    Err(e) => {
                        error!("Failed to get SSHFS options from environment: {:#?}", e);
                        exit(1);
                    }
                };
                let handler = Sshfs::new(
                    sshfs_options.mountpoint,
                    sshfs_options.connection_string,
                    sshfs_options.options,
                    sshfs_options.password,
                    sshfs_options.extra_args,
                );
                Box::new(handler)
            }
            _ => {
                error!(
                    "Protocol {:#?} is not supported as a remote filesystem",
                    program_options.protocol
                );
                exit(1);
            }
        };

    // Mount the remote filesystem using the protocol handler if necessary.
    match protocol_handler.mount().await {
        Ok(_) => {
            info!(
                "Successfully mounted filesystem at {}",
                &program_options.serve_dir
            );
        }
        Err(e) => {
            error!("Failed to mount filesystem: {:#?}", e);
            exit(1);
        }
    }

    // Setup signal handling for unmounting the remote filesystem when exiting.
    let signals: signal_hook_tokio::SignalsInfo =
        Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT]).expect("Failed to register signals");
    let handle = signals.handle();
    let signals_task = tokio::spawn(handle_unmount_on_signal(signals, protocol_handler));

    // Create the app and start it.
    let app = create_app(&program_options.serve_dir);
    info!("Serving files from {}", &program_options.serve_dir);
    start_app(app, &program_options.socket_addr).await;

    // Unregister signal handlers.
    handle.close();
    signals_task.await.unwrap();
}

/// Create the app.
fn create_app(serve_directory: &String) -> Router {
    Router::new()
        .nest_service(
            "/",
            ServeDir::new(&serve_directory)
                .append_index_html_on_directories(true)
                .fallback(handle_404_file.into_service()),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
}

/// Start the app.
async fn start_app(app: Router, socket_addr: &SocketAddr) {
    info!("Server listening on {}", socket_addr);
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Handle 404 errors by returning a 404 message.
async fn handle_404_file() -> (StatusCode, &'static str) {
    (
        StatusCode::NOT_FOUND,
        "The requested resource could not be found.",
    )
}

/// Handles unmounting the remote filesystem on a signal.
async fn handle_unmount_on_signal(
    mut signals: Signals,
    mut mount_handler: Box<dyn ProtocolHandler<'_> + Send + Sync>,
) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT | SIGQUIT | SIGHUP => match mount_handler.unmount().await {
                Ok(_) => {
                    info!("Successfully unmounted filesystem");
                    exit(0)
                }
                Err(e) => {
                    error!("Failed to unmount filesystem: {:#?}", e);
                    exit(1);
                }
            },
            _ => unreachable!(),
        }
    }
}
