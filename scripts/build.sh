#!/bin/bash
set -euo pipefail

export ARTIFACT_NAME="eks-creds-$1"

# Build for the target
cargo build --release --locked --target "$1"

# Create the artifact
echo "Working dir content:"
ls -la

echo "target dir content:"
ls -la target

echo "target/release dir content:"
ls -la target/release

cp "target/release/eks-creds" "$ARTIFACT_NAME"
cp README.md LICENSE "$ARTIFACT_NAME"

# Zip the artifact
if ! command -v zip &>/dev/null; then
    sudo apt-get update && sudo apt-get install -yq zip
fi

zip -r "$ARTIFACT_NAME.zip" "$ARTIFACT_NAME"
