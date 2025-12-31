#!/usr/bin/env bash
set -euo pipefail

# Build the Docker image
echo "Building container image..."
docker build -t steady-standard .

# Run with single CPU core
echo "Running with simulated 1 CPU core..."
docker run --rm -it \
    --cpus=1 \
    -p 9900:9900 \
    steady-standard

# Notes:
#  --rm        cleans up after exit
#  -it         allows realtime logs
#  -p 9900:9900 exposes telemetry dashboard
#  --cpus=1    simulates a machine with 1 logical CPU

# Validation:
#   1. Watch logs for “Spawning SoloAct ... on new OS thread”
#   2. Visit http://127.0.0.1:9900 while running
#   3. Confirm metrics appear and shutdown cleanly