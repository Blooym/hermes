# Hermes

A simple & lightweight file server that automatically handles remote filesystems and serves them over HTTP.

## Usage

Docker is the recommended way to run Hermes, however it can also be used as a standalone binary. Please keep in mind that if you choose to use Hermes as a standalone binary, you will need to install the dependencies for the protocol you wish to use.

It is recommended to place Hermes behind a reverse proxy that can provide a caching layer as remote filesystems can be relatively slow to request from depending on their location relative to the server and the protocol used. You will also need to use a reverse proxy if you wish to use HTTPS as Hermes does not support it natively.

### Supported Protocols

Hermes currently supports the following protocols, contributions are welcome to support more.

| Protocol | Supported |
| --- | --- |
| SSHFS | ✅ |
| Samba | ❌ |
| NFS | ❌ |
| FTP | ❌ |
| WebDAV | ❌ |

## Configuration

Hermes is configured primarily through environment variables. although some general configuration can be done through command line arguments. The following sections will detail the available configuration options.

### General

The following variables are used regardless of the protocol selected.

| Variable | Description | Flag | Default | Required |
| --- | --- | --- | --- | --- |
| `HERMES_SOCKET_ADDR` | The address to bind the HTTP server to. | `--socket-addr` | `0.0.0.0:8080` | No |
| `HERMES_MOUNT_PATH` | The path to mount the remote filesystem to. | `--mountpoint` | N/A | Yes |
| `HERMES_PROTOCOL` | The protocol to use for the remote filesystem. | `--protocol` | N/A | Yes |

### SSHFS

The following variables are used when the SSHFS protocol is selected.

| Variable | Description | Required |
| --- | --- | --- |
| `HERMES_SSHFS_CONNECTION_STRING` | The connection string to use for SSHFS. | Yes |
| `HERMES_SSHFS_PASSWORD` | The password to use for SSHFS. | Yes |
| `HERMES_SSHFS_OPTIONS` | Additional options to pass to SSHFS. | No |
| `HERMES_SSHFS_ARGS` | Additional arguments to pass to SSHFS. | No |
