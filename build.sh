#!/usr/bin/env bash
set -euo pipefail

# Ensure rust target
if ! rustc --print target-list | grep -q '^wasm32-unknown-unknown$'; then
  echo "Adding Rust target wasm32-unknown-unknown..."
  rustup target add wasm32-unknown-unknown
fi

# Ensure tools on PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Ensure wasm-bindgen CLI exists
if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "Installing wasm-bindgen-cli (0.2.100) ..."
  cargo install -f wasm-bindgen-cli --version 0.2.100
fi

# Ensure wasm-opt exists
if ! command -v wasm-opt >/dev/null 2>&1; then
  echo "Error: wasm-opt not found. Install Binaryen (brew install binaryen | apt install binaryen | choco install binaryen | npm i -g binaryen)."
  exit 1
fi

echo "Compiling Rust to WASM..."
cargo build --target wasm32-unknown-unknown --release

OUT_DIR="./video-editor/src/wasm"
mkdir -p "$OUT_DIR"

echo "Generating JS bindings..."
wasm-bindgen target/wasm32-unknown-unknown/release/video_processor.wasm \
  --out-dir "$OUT_DIR" \
  --target web \
  --no-typescript

echo "Optimizing WASM..."
wasm-opt -O4 "$OUT_DIR/video_processor_bg.wasm" -o "$OUT_DIR/video_processor_bg.wasm"

echo "Done."
