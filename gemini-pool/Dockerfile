# ==============================================================================
# Builder Stage
# Use the official Rust image to build the application.
# ==============================================================================
FROM rust:1-slim-bullseye AS builder

# Create a new empty shell project to cache dependencies.
WORKDIR /usr/src/app
RUN USER=root cargo new --bin gemini-pool
WORKDIR /usr/src/app/gemini-pool

# Copy over the dependency manifests.
COPY gemini-pool/Cargo.toml gemini-pool/Cargo.lock ./

# Install build dependencies for OpenSSL.
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Build only the dependencies to cache them.
RUN cargo build --release
RUN rm src/*.rs

# Copy the actual source code.
COPY ./gemini-pool/src ./src

# Build the application.
# This will use the cached dependencies if the source code has not changed.
RUN rm ./target/release/deps/gemini_pool*
RUN cargo build --release

# ==============================================================================
# Final Stage
# Use a slim, secure base image for the final container.
# We use debian:11-slim to match the builder's OS (bullseye) and ensure
# compatibility with linked libraries like OpenSSL.
# ==============================================================================
FROM debian:11-slim

# Install runtime dependencies (CA certificates for HTTPS requests).
# Clean up apt cache to keep the image small.
RUN apt-get update && \
    apt-get install -y ca-certificates --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory.
WORKDIR /usr/local/bin

# Copy the compiled binary from the builder stage.
COPY --from=builder /usr/src/app/gemini-pool/target/release/gemini-pool .

# Expose the port the app runs on.
EXPOSE 8080

# Set the entrypoint.
CMD ["./gemini-pool"]