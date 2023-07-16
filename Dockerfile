# -----------
#    BUILD   
# -----------
FROM rust:1-alpine as build
WORKDIR /build

# Build dependencies.
RUN apk add --no-cache --update build-base

# Build the project.
COPY ["Cargo.toml", "Cargo.lock", "./"]
COPY src/ src/
RUN cargo build --release --bin hermes


# -----------
#   RUNTIME  
# -----------
FROM alpine as runtime
WORKDIR /app
MAINTAINER "Blooym"

# Runtime dependencies.
RUN apk add --no-cache --update sshfs fuse

# Create a user to run the app.
RUN addgroup -S hermes \
    && adduser -s /bin/false -S -G hermes -H -D hermes -h /app \
    && chown -R hermes:hermes /app
USER hermes

# Setup default configuration.
RUN mkdir -p /app/servefs
ENV HERMES_SERVE_DIR=/app/servefs
ENV HERMES_SSHFS_MOUNTPOINT=/app/servefs
ENV HERMES_SOCKETADDR=0.0.0.0:8080
ENV RUST_LOG=info
EXPOSE 8080

# Copy the binary from the build stage.
COPY --from=build /build/target/release/hermes /app/bin/hermes

# Run the app
CMD ["/app/bin/hermes"]