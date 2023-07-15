# -----------
#    BUILD   
# -----------
FROM rust:1-alpine as build
WORKDIR /build

# Install essential build tools.
RUN apk add --no-cache --update build-base

# Build the binary.
COPY ["Cargo.toml", "Cargo.lock", "./"]
COPY src/ src/
RUN cargo build --release --bin hermes


# -----------
#   RUNTIME  
# -----------
FROM alpine as runtime
WORKDIR /app

# Install dependencies.
RUN apk add --no-cache --update sshfs fuse

# Setup and switch to non-root user.
RUN addgroup -S hermes \
    && adduser -s /bin/false -S -G hermes -H -D hermes -h /app \
    && chown -R hermes:hermes /app
USER hermes

# Setup socket address.
ENV HERMES_SOCKETADDR=0.0.0.0:8000
EXPOSE 8000

# Setup remote mountpoint.
RUN mkdir -p /app/remotefs
ENV HERMES_MOUNT_PATH=/app/remotefs

# Grab binary from build stage.
COPY --from=build /build/target/release/hermes /app/bin/hermes

# Run the binary.
CMD ["/app/bin/hermes"]