#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output_dir="$project_root/dist"
wasm_target="$project_root/target/wasm32-unknown-unknown/release"
web_prefix="${AETHER_WEB_PREFIX:-}"

if [[ "${1:-}" == "--prefix" ]]; then
  web_prefix="${2:-}"
fi
web_prefix="/${web_prefix#/}"
web_prefix="${web_prefix%/}"
if [[ "$web_prefix" == "/" ]]; then
  web_prefix=""
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen is required: install the version matching Cargo.lock" >&2
  exit 1
fi

rm -rf "$output_dir"
mkdir -p "$output_dir"
cargo build --manifest-path "$project_root/Cargo.toml" --release --target wasm32-unknown-unknown -p lobby_web

sed "s|__AETHER_PREFIX__|$web_prefix|g" "$project_root/web/lobby.html" > "$output_dir/index.html"
cp "$project_root/web/style.css" "$output_dir/style.css"
sed "s|__AETHER_PREFIX__|$web_prefix|g" "$project_root/web/lobby-loader.js" > "$output_dir/lobby-loader.js"
sed "s|__AETHER_PREFIX__|$web_prefix|g" "$project_root/web/games.json" > "$output_dir/games.json"
mkdir -p "$output_dir/game/runner" "$output_dir/game/shipwright" "$output_dir/game/sandbox"
cp "$output_dir/index.html" "$output_dir/game/runner/index.html"
cp "$output_dir/index.html" "$output_dir/game/shipwright/index.html"
cp "$output_dir/index.html" "$output_dir/game/sandbox/index.html"
mkdir -p "$output_dir/assets"
cp -R "$project_root/assets/." "$output_dir/assets/"
wasm-bindgen "$wasm_target/lobby_web.wasm" --out-dir "$output_dir" --target web --no-typescript

echo "Web build ready in $output_dir (prefix: ${web_prefix:-/})"
echo "Serve it with: python3 -m http.server --directory $output_dir 8080"
