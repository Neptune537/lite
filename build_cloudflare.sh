# Cloudflare Pages Build Script for HeavenlyBound Lite (Wasm + WebGPU + tr7n7y7 Kuroshio Studio)
set -e

if [ -d "../heavenlyBound" ]; then
  echo "=== Installing Rust WebAssembly Target ==="
  rustup target add wasm32-unknown-unknown

  echo "=== Installing Trunk ==="
  cargo install --locked trunk

  echo "=== Building HeavenlyBound Lite for Cloudflare Pages ==="
  trunk build --release
else
  echo "=== Sibling ../heavenlyBound absent; packaging tr7n7y7 Lite & Kuroshio Studio static dist/ ==="
  mkdir -p dist
  cp -r index.html office.html platform-auth.js club-comms.js station-radio.js sonar-client.js auth-config.json sonar-config.json manifest.webmanifest _headers app-icons dist/
fi

echo "=== Build Complete! Output directory: dist/ ==="
