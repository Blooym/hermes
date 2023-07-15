use std::net::SocketAddr;

use clap::Parser;
use dotenv::dotenv;
use remote_mount::protocols::Protocols;

#[derive(Parser, Debug)]
#[clap(
    name = "hermes",
    author = "Blooym",
    about = "A simple & lightweight file server that automatically handles remote filesystems and serves them over HTTP."
)]
pub struct ProgramOptions {
    /// The address + port to listen on.
    #[clap(
        short,
        long,
        default_value = "0.0.0.0:8080",
        env = "HERMES_SOCKET_ADDR"
    )]
    pub socket_addr: SocketAddr,

    /// The protocol to use for the remote filesystem.
    #[clap(short = 'p', long, env = "HERMES_PROTOCOL")]
    pub protocol: Protocols,

    /// Where to mount the remote filesystem to.
    #[clap(short = 'm', long, env = "HERMES_MOUNT_PATH")]
    pub mountpoint: String,
}

impl ProgramOptions {
    pub fn from_env_and_args() -> Self {
        dotenv().ok();
        Self::parse()
    }
}
