#!/usr/bin/env bash
# Rebuilds the committed standard-library wasm fixture used by the runtime contract.
set -euo pipefail

cd "$(dirname "$0")/../fixtures/rust-std"

if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    rustup target add wasm32-unknown-unknown
fi

cargo build --release --target wasm32-unknown-unknown --target-dir ./target
cp ./target/wasm32-unknown-unknown/release/rust_std.wasm ./rust_std.wasm

echo "wrote fixtures/rust-std/rust_std.wasm"
