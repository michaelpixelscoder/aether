#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output_dir="$project_root/dist"
wasm_target="$project_root/target/wasm32-unknown-unknown/release"

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen is required: install the version matching Cargo.lock" >&2
  exit 1
fi

cargo build --manifest-path "$project_root/Cargo.toml" --release --target wasm32-unknown-unknown -p lobby_web -p sandbox -p shipwright

mkdir -p "$output_dir/game/sandbox"
mkdir -p "$output_dir/game/shipwright"
cp "$project_root/web/lobby.html" "$output_dir/index.html"
cp "$project_root/web/game.html" "$output_dir/game/sandbox/index.html"
cp "$project_root/web/shipwright.html" "$output_dir/game/shipwright/index.html"
cp "$project_root/web/style.css" "$output_dir/style.css"

wasm-bindgen "$wasm_target/lobby_web.wasm" --out-dir "$output_dir" --target web --no-typescript
wasm-bindgen "$wasm_target/sandbox.wasm" --out-dir "$output_dir/game/sandbox" --target web --no-typescript
wasm-bindgen "$wasm_target/shipwright.wasm" --out-dir "$output_dir/game/shipwright" --target web --no-typescript

echo "Web build ready in $output_dir"
echo "Serve it with: python3 -m http.server --directory $output_dir 8080"
