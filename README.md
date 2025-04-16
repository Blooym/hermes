# Hermes

Named after the Greek god of travel, Hermes is a simple & lightweight file server that can automatically handle both local and remote filesystems using a variety of protocols.

> [!CAUTION]  
> **This project is made for me and my needs and no support will be offered to anyone trying to use it.** 
>
> Breaking changes CAN and WILL be made at any time; the purpose of the software may also change.

## Features

### Supported Protocols

Hermes currently supports the following protocols, contributions are welcome to add more protocols.

| Protocol | Supported |
| --- | --- |
| Local | ✅ |
| SSHFS | ✅ |

## Usage

A container is the recommended way to run Hermes, it can however be used as a standalone binary although it must be manually compiled and no support is offered for Windows (not including WSL). Please keep in mind that if you choose to use Hermes as an uncontained binary you will need to install the dependencies for the protocol you wish to use and handle things manually.

It is recommended to place Hermes behind a reverse proxy that can provide caching, TLS, and compression as remote filesystems can be relatively slow to request from depending on their location relative to the server and the protocol used.

### Container

A prebuilt container image with support for all protocols can be pulled from the GitHub Container Registry - you can view the available tags [here](https://github.com/Blooym/hermes/pkgs/container/hermes/versions?filters%5Bversion_type%5D=tagged).

This container image will automatically set a few environment variables to ensure it works out of the box. It is recommended you do not override the default variables as it may lead to unexpected behaviour. The following variables are set by the container image:

| Variable | Value | Reason |
| --- | --- | --- |
| `HERMES_SOCKET_ADDR` | `0.0.0.0:8080` | To `0.0.0.0` allows the container to be accessed from outside networks. |
| `HERMES_SERVE_DIR` | `/app/servefs` | It has already been created inside of the container with the correct permissions. |
| `HERMES_SSHFS_MOUNTPOINT` | `/app/servefs` | To mount to the same directory as the serve directory |
| `RUST_LOG` | `INFO` | To provide more information about the state of the server |

Please note that the container will require `CAP_SYS_ADMIN` and access to `/dev/fuse` if you wish to mount remote filesystems, you can grant this by passing the `--device=/dev/fuse --cap-add=SYS_ADMIN` flags to the run command or equivilant values in your compose file.

## Configuration

Hermes is configured primarily through environment variables, although some general configuration can be done through command line arguments. The following sections will detail the available configuration options.

### General

The following variables are used regardless of the protocol selected.

| Variable | Flag | Description  | Default | Required |
| --- | --- | --- | --- | --- |
| `HERMES_SOCKET_ADDR` | `--socket-addr` | The address to bind the HTTP server to. | `0.0.0.0:8080` | YES |
| `HERMES_SERVE_DIR` | `--serve-dir` | The directory to serve files from. | N/A | YES |
| `HERMES_PROTOCOL` | `--protocol` | The protocol to use for the remote filesystem. | N/A | YES |
| `RUST_LOG` | N/A | The log level to use for tracing. | N/A | NO |
| `RUST_BACKTRACE` | N/A | Whether to enable backtraces for errors. | N/A | NO |

### SSHFS

The following variables are used when the SSHFS protocol is selected.

| Variable | Description | Required |
| --- | --- | --- |
| `HERMES_SSHFS_CONNECTION_STRING` | The connection string to use for SSHFS. | YES |
| `HERMES_SSHFS_MOUNTPOINT` | The mountpoint to use for SSHFS. | YES |
| `HERMES_SSHFS_PASSWORD` | The password to use for SSHFS. | YES |
| `HERMES_SSHFS_OPTIONS` | Additional options to pass to SSHFS. | NO |
| `HERMES_SSHFS_ARGS` | Additional arguments to pass to SSHFS. | NO |
