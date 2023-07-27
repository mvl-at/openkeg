#!/usr/bin/env sh
# Script Name: docker-multiarch-build.sh
# Description: This script builds the 'openkeg' application for the specified architecture.
# Author: Richard Stöckl
# Date: July 27, 2023

# Usage: ./docker-multiarch-build.sh
# This script requires the following environment variables to be set:
# - TARGETARCH: The target architecture for the build (arm64 or amd64). Normally set by BuildKit.

echo "Build for $TARGETOS/$TARGETPLATFORM on $BUILDOS/$BUILDPLATFORM"

# Check the target architecture and set appropriate Rust platform and linker
case "$TARGETARCH" in
arm64)
  RUSTPLATFORM="aarch64-unknown-linux-musl"
  RUSTLINKER="aarch64-none-elf-gcc"
  ;;
amd64)
  RUSTPLATFORM="x86_64-unknown-linux-musl"
  RUSTLINKER="x86_64-alpine-linux-musl-gcc"
  ;;
*)
  echo "Error: Unknown architecture specified: '$TARGETARCH'"
  exit 1
esac

echo "Use rust platform $RUSTPLATFORM and linker $RUSTLINKER"

# Build the 'openkeg' application using Cargo with the specified Rust platform and linker
cargo build --target=$RUSTPLATFORM --release --config target.$RUSTPLATFORM.linker=\"$RUSTLINKER\"

# Create a directory for release artifacts and copy the built binary to it
mkdir -p "/target/release"
cp "target/$RUSTPLATFORM/release/openkeg" "/target/release/openkeg.$TARGETARCH"

# List the contents of the /workspace/target/release directory
# This is just for verification purposes
ls /workspace/target/release
