#![forbid(unsafe_code)]

mod program_options;

use crate::program_options::ProgramOptions;
use axum::{handler::HandlerWithoutStateExt, http::StatusCode, Router};
use dotenv::dotenv;
use futures::stream::StreamExt;
use remote_mount::protocols::{sshfs::Sshfs, FromEnv, ProtocolHandler, Protocols};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::process::exit;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_http::trace::{self};
use tracing::{error, info, Level};

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Setup tracing from the environment.
    tracing_subscriber::fmt::init();

    // Get the program options.
    let options: ProgramOptions = ProgramOptions::from_env_and_args();

    // Get the protocol handler from the options.
    info!("Using protocol '{:#?}'", options.protocol);
    let mut protocol_handler: Box<dyn ProtocolHandler + Send + Sync> = match options.protocol {
        Protocols::Sshfs => {
            let handler = match Sshfs::with_mountpoint_from_env(options.mountpoint.clone()) {
                Ok(h) => h,
                Err(e) => {
                    error!("Failed to create sshfs protocol handler: {:#?}", e);
                    exit(1);
                }
            };
            Box::new(handler)
        }
    };

    // Mount the remote filesystem using the protocol handler if necessary.
    match protocol_handler.mount().await {
        Ok(_) => {
            info!(
                "Successfully mounted filesystem, available at {}",
                options.mountpoint
            );
        }
        Err(e) => {
            error!("Failed to mount filesystem: {:#?}", e);
            exit(1);
        }
    }

    // Create the app.
    let app: Router = Router::new()
        .nest_service(
            "/",
            ServeDir::new(options.mountpoint.clone())
                .append_index_html_on_directories(true)
                .fallback(handle_404_file.into_service()),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    // Setup signal handling for unmounting the remote filesystem when exiting.
    let signals: signal_hook_tokio::SignalsInfo =
        Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT]).expect("Failed to register signals");
    let handle = signals.handle();
    let signals_task = tokio::spawn(handle_unmount_on_signal(signals, protocol_handler));

    // Run the server.
    info!("Server listening on {}", options.socket_addr);
    axum::Server::bind(&options.socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    // Unregister signal handlers.
    handle.close();
    signals_task.await.unwrap();
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

/// Handle 404 errors by returning a 404 message.
async fn handle_404_file() -> (StatusCode, &'static str) {
    (
        StatusCode::NOT_FOUND,
        "The requested resource could not be found.",
    )
}
