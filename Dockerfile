# ==============================================================================
# Dockerfile for SteadyState Standard
# This version copies both the standard project and its local steady_state core.
# ==============================================================================

# ---- Build Stage -------------------------------------------------------------
FROM rustlang/rust:nightly-slim AS builder

ENV RUST_BACKTRACE=1
ENV RUST_LOG=info

# Install build dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates pkg-config libssl-dev curl build-essential && \
    rm -rf /var/lib/apt/lists/*

# Set a working directory for the project
WORKDIR /app

# --------------------------------------------------------------------
# Copy both the standard project AND its dependency (core)
# NOTE:  the build context must include both directories, so you will
# run docker build from the parent folder (see command below).
# --------------------------------------------------------------------
COPY steady-state-stack/core /steady-state-stack/core
COPY steady-state-standard /app

# Build the release binary
RUN cargo build --release

# ---- Runtime Stage -----------------------------------------------------------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary out of the builder image
COPY --from=builder /app/target/release/standard /usr/local/bin/standard

# Working directory for logs or mounts
WORKDIR /app

# Expose telemetry port
EXPOSE 9900

# Default command (can be overridden)
CMD ["standard", "--rate", "200", "--beats", "1000"]