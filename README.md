# Hermes

> [!CAUTION]  
> **This project is made for me and my needs and no support will be offered to anyone trying to use it.** 
>
> Breaking changes CAN and WILL be made at any time; the purpose of the software may also change.

Hermes is a simple & lightweight fileserver with support from serving from a variety of storage backends.

## Usage

The recommended way to run Hermes is via Docker as it provides an optimal pre-configured environment for every supported storage backend. Hermes can also be ran as a standalone binary - although it must be manually compiled and you are responsible for configuring the runtime environment to work with your desired storage backend.

### Docker

A premade Dockerfile support for all protocols is available [here](./Dockerfile). **Please note that the container will require `CAP_SYS_ADMIN` and access to `/dev/fuse` if you wish to mount remote filesystems via FUSE**, and may also require additional changes to SELinux/AppArmor configurations.

## Configuration

Hermes is configured via command-line flags or environment variables and has full support for loading from `.env` files. Below is a list of all supported configuration options. You can also run `hermes --help` to get up-to-date information including default values.

| Environment                     | Flag                       | Description                                                                                                                                                               | Default        |
| ------------------------------- | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| `HERMES_SOCKET_ADDR`            | `--address`                | The address to bind the HTTP server to.                                                                                                                                   | `0.0.0.0:8080` |
| `HERMES_STORAGE_BACKEND`        | `--storage-backend`        | The storage backend to serve files from.                                                                                                                                  | N/A            |
| `HERMES_FILE_CACHE_DURATION`    | `--file-cache-duration`    | The duration of time to cache files for. Files will not be revalidated by the client during this time.                                                                    | N/A            |
| `HERMES_FILE_STREAM_BUFFERSIZE` | `--file-stream-buffersize` | The buffer size (in bytes) to use when streaming files from storage. Larger sizes may result in quicker file loads at the cost of increased memory usage for large files. | `64000 bytes`  |
| `RUST_LOG`                      | N/A                        | The log level to use for tracing.                                                                                                                                         | `info`         |

### Storage Backends

#### Local Filesystem

Enabled by passing `--storage-backend=fs://<base_path>`.

There is no additional configuration for this backend.

#### SSHFS

Enabled by passing `--storage-backend=sshfs://<mountpoint_path>`.

Please note that you may have to add `StrictHostKeyChecking=no` to `SSHFS_OPTIONS` if you do not already have the server host stored in `known_hosts` as otherwise the connection will hang waiting for the client to accept the key.

| Variable                  | Description                                                                  | Required |
| ------------------------- | ---------------------------------------------------------------------------- | -------- |
| `SSHFS_CONNECTION_STRING` | The connection string to use for SSHFS.                                      | YES      |
| `SSHFS_MOUNTPOINT`        | The mountpoint to use for SSHFS.                                             | YES      |
| `SSHFS_PASSWORD`          | The password to use for SSHFS (piped via stdin), optional if using SSH keys. | NO       |
| `SSHFS_OPTIONS`           | Additional options to pass to SSHFS on mount.                                | NO       |

#### S3

Enabled by passing `--storage-backend=s3://<bucket_name>`.

Configuration and credentials for this backend is handled via the [AWS credential provider chain](https://docs.aws.amazon.com/sdkref/latest/guide/standardized-credentials.html), please refer to the AWS S3 documentation for a guide on configuring S3 via your chosen provider.