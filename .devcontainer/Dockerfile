FROM rust:1-bookworm

# Install dependencies
RUN apt-get update && apt-get install -y sshfs fuse3

# Add non-root user
RUN adduser --disabled-password --gecos "" dev

# Install Rust components
RUN rustup component add rustfmt clippy