# Hermes
[![Code Changes](https://github.com/Blooym/Hermes/actions/workflows/code_changes.yml/badge.svg)](https://github.com/Blooym/Hermes/actions/workflows/code_changes.yml)
[![Container Changes](https://github.com/Blooym/Hermes/actions/workflows/container_changes.yml/badge.svg)](https://github.com/Blooym/Hermes/actions/workflows/container_changes.yml)

A simple & lightweight file server that automatically handles remote filesystems and serves them over HTTP.

## Usage

Docker is the recommended way to run Hermes, however it can also be used as a standalone binary. Please keep in mind that if you choose to use Hermes as a standalone binary, you will need to install the dependencies for the protocol you wish to use.

It is recommended to place Hermes behind a reverse proxy that can provide a caching layer as remote filesystems can be relatively slow to request from depending on their location relative to the server and the protocol used. You will also need to use a reverse proxy if you wish to use HTTPS as Hermes does not support it natively.

### Docker

A prebuilt Docker image with all protocols enabled can be pulled from the GitHub Container Registry. The latest version is available at
```
FROM ghcr.io/blooym/hermes:latest
```

You can also use a specific version by replacing `latest` with the version you wish to use. A list of available versions can be found [here](https://github.com/Blooym/hermes/pkgs/container/hermes/versions?filters%5Bversion_type%5D=tagged).

This image will automatically set a default mountpoint inside the container and a socket address of `0.0.0.0:8080` which can be overridden by setting the appropriate environment variables. You will need to pass the `--privileged` flag to the container to allow it to mount the remote filesystem.

## Supported Protocols

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
| `RUST_LOG` | The log level to use for messages. | N/A | `ERROR` | No |

### SSHFS

The following variables are used when the SSHFS protocol is selected.

| Variable | Description | Required |
| --- | --- | --- |
| `HERMES_SSHFS_CONNECTION_STRING` | The connection string to use for SSHFS. | Yes |
| `HERMES_SSHFS_PASSWORD` | The password to use for SSHFS. | Yes |
| `HERMES_SSHFS_OPTIONS` | Additional options to pass to SSHFS. | No |
| `HERMES_SSHFS_ARGS` | Additional arguments to pass to SSHFS. | No |
