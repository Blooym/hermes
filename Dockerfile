# -----------
#    BUILD
# -----------
FROM rust:1-alpine AS build
WORKDIR /build
RUN apk add --no-cache --update build-base

# Pre-cache dependencies
COPY ["Cargo.toml", "Cargo.lock", "./"]
RUN mkdir src \
    && echo "// Placeholder" > src/lib.rs \
    && cargo build --release \
    && rm src/lib.rs

# Build
COPY src ./src
RUN cargo build --release

# -----------
#   RUNTIME  
# -----------
FROM alpine
WORKDIR /app

# Runtime dependencies.
RUN apk add --no-cache --update sshfs fuse

# Create a user to run the app.
RUN addgroup -S hermes \
    && adduser -s /bin/false -S -G hermes -H -D hermes

# Setup the default configuration for the container.
RUN mkdir -p /srv/dollhouse && chown -R hermes:hermes /srv/dollhouse
ENV HERMES_ADDRESS=0.0.0.0:8080
ENV RUST_LOG=info
EXPOSE 8080

# Copy the binary from the build stage.
COPY --from=build /build/target/release/hermes /usr/bin/hermes

# Run the app as a non-root user.
USER hermes
ENTRYPOINT ["/usr/bin/hermes"]