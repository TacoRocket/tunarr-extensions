#!/bin/sh
set -eu
cd "$(dirname "$0")"
export RUSTFLAGS="-C link-arg=--import-memory --remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}/registry/src=/registry --remap-path-prefix=$PWD=compute"
cargo build --locked --target wasm32-unknown-unknown --release
cp "${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/potion_compute.wasm" ../potion.wasm
