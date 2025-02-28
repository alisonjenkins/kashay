#!/bin/bash
set -euo pipefail

export ARTIFACT_NAME="eks-creds-$1"
mkdir -p "$ARTIFACT_NAME"

# Build for the target
cargo build --release --locked --target "$1"

# Create the artifact
cp "target/$1/release/eks-creds" "$ARTIFACT_NAME"
cp README.md LICENSE "$ARTIFACT_NAME"

# Zip the artifact
if ! command -v zip &>/dev/null; then
    sudo apt-get update && sudo apt-get install -yq zip
fi

zip -r "$ARTIFACT_NAME.zip" "$ARTIFACT_NAME"
