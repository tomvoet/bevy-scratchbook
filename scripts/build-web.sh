#!/usr/bin/env bash
# Builds into site/dist. Needs the wasm32 target and the wasm-bindgen-cli
# version pinned in Cargo.lock.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build --release --target wasm32-unknown-unknown -p sims
wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
  --out-dir site/dist --out-name sims \
  target/wasm32-unknown-unknown/release/sims.wasm

if command -v wasm-opt >/dev/null; then
  wasm-opt -O3 -o site/dist/sims_bg.wasm site/dist/sims_bg.wasm
fi
