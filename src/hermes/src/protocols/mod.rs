pub mod sshfs;

use self::sshfs::SshfsOptions;
use crate::{env::FromEnv, program_options::Protocols};
use remote_mount::protocols::ProtocolHandler;
use std::process::exit;
use tracing::error;

/// Get the protocol handler for the given protocol.
pub fn get_protocol_handler<'r>(
    protocol: &Protocols,
) -> Box<dyn ProtocolHandler<'r> + Send + Sync> {
    let handler = match protocol {
        Protocols::Sshfs => match SshfsOptions::from_env() {
            Ok(options) => options,
            Err(e) => {
                error!("Failed to get SSHFS options from environment: {:#?}", e);
                exit(1);
            }
        }
        .create_handler_from_opts(),
        _ => {
            error!(
                "Protocol {:#?} is not supported as a remote filesystem",
                protocol
            );
            exit(1);
        }
    };
    Box::new(handler)
}
