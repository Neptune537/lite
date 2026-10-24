# Cloudflare Pages Build Script for HeavenlyBound Lite (Wasm + WebGPU)
set -e

echo "=== Installing Rust WebAssembly Target ==="
rustup target add wasm32-unknown-unknown

echo "=== Installing Trunk ==="
cargo install --locked trunk

echo "=== Building HeavenlyBound Lite for Cloudflare Pages ==="
trunk build --release

echo "=== Build Complete! Output directory: dist/ ==="
