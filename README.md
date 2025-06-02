# Hermes

Hermes is a simple & lightweight file server that can serve files from from a variety of storage backends.

> [!CAUTION]  
> **This project is made for me and my needs and no support will be offered to anyone trying to use it.** 
>
> Breaking changes CAN and WILL be made at any time; the purpose of the software may also change.

## Features

### Supported Storage Backends

- Filesystem
- SSHFS (via sshfs-fuse)
- S3

## Usage

The recommended way to run Hermes is via Docker, it can however be used as a standalone binary - although it must be manually compiled. Please keep in mind that if you choose to use Hermes as an uncontained binary you will need to install the dependencies for the backend you wish to use manually.

### Docker

A premade Dockerfile support for all protocols is available [here](./Dockerfile). **Please note that the container will require `CAP_SYS_ADMIN` and access to `/dev/fuse` if you wish to mount remote filesystems via FUSE**.

## Configuration

Hermes is configured via command-line flags or environment variables and has full support for loading from `.env` files. Below is a list of all supported configuration options. You can also run `hermes --help` to get an up-to-date including default values.

| Environment                 | Flag                   | Description                                                                                            | Default        | Required |
| --------------------------- | ---------------------- | ------------------------------------------------------------------------------------------------------ | -------------- | -------- |
| `HERMES_SOCKET_ADDR`        | `--address`            | The address to bind the HTTP server to.                                                                | `0.0.0.0:8080` | YES      |
| `HERMES_STORAGE_BACKEND`    | `--storage-backend`    | The storage backend to serve files from.                                                               | N/A            | YES      |
| `HERMES_FILE_CACHE_DURATION | `--file-cache-duration | The duration of time to cache files for. Files will not be revalidated by the client during this time. | `1 minute      | NO       |
| `RUST_LOG`                  | N/A                    | The log level to use for tracing.                                                                      | `info`         | NO       |

### SSHFS Backend

| Variable                  | Description                                                                  | Required |
| ------------------------- | ---------------------------------------------------------------------------- | -------- |
| `SSHFS_CONNECTION_STRING` | The connection string to use for SSHFS.                                      | YES      |
| `SSHFS_MOUNTPOINT`        | The mountpoint to use for SSHFS.                                             | YES      |
| `SSHFS_PASSWORD`          | The password to use for SSHFS (piped via stdin), optional if using SSH keys. | NO       |
| `SSHFS_OPTIONS`           | Additional options to pass to SSHFS on mount.                                | NO       |

### S3 Backend

Configuration is handled via the [AWS credential provider chain](https://docs.aws.amazon.com/sdkref/latest/guide/standardized-credentials.html), please refer to the AWS S3 documentation for a guide on configuring S3 via your chosen provider.