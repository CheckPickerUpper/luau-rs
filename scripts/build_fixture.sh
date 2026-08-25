#!/usr/bin/env bash
# Rebuilds the committed wasm fixture that the integration suite decodes.
#
# The fixture is a tiny cdylib crate (fixtures/rust-hello) compiled to
# wasm32-unknown-unknown with a size-optimized, panic-abort release profile.
# The committed binary is what tests/round_trip.rs decodes and validates
# against the official Luau tools, so the binary must be refreshed whenever
# the crate source changes.
set -euo pipefail

cd "$(dirname "$0")/../fixtures/rust-hello"

if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    rustup target add wasm32-unknown-unknown
fi

cargo build --release --target wasm32-unknown-unknown --target-dir ./target

cp ./target/wasm32-unknown-unknown/release/rust_hello.wasm ./rust_hello.wasm

echo "wrote fixtures/rust-hello/rust_hello.wasm"
