#![forbid(unsafe_code)]

use axum::{handler::HandlerWithoutStateExt, http::StatusCode, Router};
use clap::Parser;
use dotenv::dotenv;
use futures::stream::StreamExt;
use remote_mount::sshfs::SshfsHandler;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::{io::Error, net::SocketAddr, process::exit};
use tower_http::services::ServeDir;

#[derive(Parser, Debug)]
#[clap(
    name = "hermes",
    author = "Blooym",
    about = "A simple & lightweight file server that handles and serves from remote mountpoints"
)]
struct Args {
    /// The address + port to listen on.
    #[clap(
        short,
        long,
        default_value = "0.0.0.0:8080",
        env = "HERMES_SOCKET_ADDR"
    )]
    pub socket_addr: SocketAddr,

    /// Where to mount the remote filesystem to.
    #[clap(short = 'm', long, env = "HERMES_REMOTE_MOUNTPOINT")]
    pub remote_mountpoint: String,

    /// The connection string for sshfs.
    #[clap(short = 'c', long, env = "HERMES_SSHFS_CONNECTION_STRING")]
    pub sshfs_connection_string: String,

    /// The password for sshfs, will be passed via stdin.
    #[clap(short = 'p', long, env = "HERMES_SSHFS_PASSWORD")]
    pub sshfs_password: String,

    /// Options to pass to sshfs.
    #[clap(short = 'o', long, env = "HERMES_SSHFS_OPTIONS", default_value = "")]
    pub sshfs_options: String,

    /// Additional arguments to pass to sshfs.
    #[clap(short = 'a', long, env = "HERMES_SSHFS_ARGS", default_value = "")]
    pub sshfs_args: String,
}

impl Args {
    /// Parse the options from the environment and command line arguments.
    pub fn from_env_and_args() -> Self {
        dotenv().ok();
        Self::parse()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = Args::from_env_and_args();

    // Create handler for sshfs.
    let mut sshfs_handler = SshfsHandler::new(
        config.sshfs_connection_string.clone(),
        config.remote_mountpoint.clone(),
        config.sshfs_password.clone(),
        config.sshfs_options.clone(),
        config.sshfs_args.clone(),
    );

    // Mount the remote filesystem.
    match sshfs_handler.mount().await {
        Ok(_) => println!("Mounted remote filesystem."),
        Err(e) => {
            eprintln!("Failed to mount remote filesystem: {:?}", e);
            exit(1);
        }
    }

    // Handle signals.
    let signals = Signals::new([SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
    let handle = signals.handle();
    let signals_task = tokio::spawn(handle_unmount_on_signal(signals, sshfs_handler));

    // Create the app.
    let app: Router = Router::new().nest_service(
        "/",
        ServeDir::new(config.remote_mountpoint.clone())
            .append_index_html_on_directories(true)
            .fallback(handle_404_file.into_service()),
    );

    // Run the server.
    println!("Server listening on {}", config.socket_addr);
    axum::Server::bind(&config.socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    // Unregister the signal handlers on exit.
    handle.close();
    signals_task.await.unwrap();

    Ok(())
}

/// Handles missing files.
async fn handle_404_file() -> (StatusCode, &'static str) {
    (
        StatusCode::NOT_FOUND,
        "The requested resource could not be found.",
    )
}

/// Handles unmounting the remote filesystem on a signal.
async fn handle_unmount_on_signal(mut signals: Signals, mut mount_handler: SshfsHandler) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT | SIGQUIT | SIGHUP => match mount_handler.unmount().await {
                Ok(_) => {
                    println!("Successfully unmounted remote filesystem.");
                    exit(0)
                }
                Err(e) => {
                    eprintln!("Failed to unmount remote filesystem: {:?}", e);
                    exit(1);
                }
            },
            _ => unreachable!(),
        }
    }
}
