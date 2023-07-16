use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use dotenv::dotenv;

#[derive(Parser, Debug)]
#[clap(
    name = "hermes",
    author = "Blooym",
    about = "A simple & lightweight file server that can automatically handle both local and remote filesystems using a variety of protocols."
)]
pub struct ProgramOptions {
    /// The address + port to listen on.
    #[clap(
        short = 's',
        long = "socket-addr",
        default_value = "0.0.0.0:8080",
        env = "HERMES_SOCKET_ADDR"
    )]
    pub socket_addr: SocketAddr,

    /// The protocol to use for the remote filesystem.
    #[clap(short = 'p', long = "protocol", env = "HERMES_PROTOCOL")]
    pub protocol: Protocols,

    // Where to serve files from.
    #[clap(short = 'd', long = "serve-dir", env = "HERMES_SERVE_DIR")]
    pub serve_dir: String,
}

impl ProgramOptions {
    pub fn from_env_and_args() -> Self {
        dotenv().ok();
        Self::parse()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocols {
    Sshfs,
    Local,
}

impl FromStr for Protocols {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sshfs" => Ok(Self::Sshfs),
            "local" => Ok(Self::Local),
            _ => Err(format!("Invalid protocol: {}", s)),
        }
    }
}
