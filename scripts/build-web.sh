#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output_dir="$project_root/dist"
wasm_target="$project_root/target/wasm32-unknown-unknown/release"

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen is required: install the version matching Cargo.lock" >&2
  exit 1
fi

rm -rf "$output_dir"
mkdir -p "$output_dir"
cargo build --manifest-path "$project_root/Cargo.toml" --release --target wasm32-unknown-unknown -p lobby_web

cp "$project_root/web/lobby.html" "$output_dir/index.html"
cp "$project_root/web/style.css" "$output_dir/style.css"
cp "$project_root/web/lobby-loader.js" "$output_dir/lobby-loader.js"
cp "$project_root/web/games.json" "$output_dir/games.json"
mkdir -p "$output_dir/game/runner" "$output_dir/game/shipwright" "$output_dir/game/sandbox"
cp "$output_dir/index.html" "$output_dir/game/runner/index.html"
cp "$output_dir/index.html" "$output_dir/game/shipwright/index.html"
cp "$output_dir/index.html" "$output_dir/game/sandbox/index.html"
mkdir -p "$output_dir/assets"
cp -R "$project_root/assets/." "$output_dir/assets/"
wasm-bindgen "$wasm_target/lobby_web.wasm" --out-dir "$output_dir" --target web --no-typescript

echo "Web build ready in $output_dir"
echo "Serve it with: python3 -m http.server --directory $output_dir 8080"
