#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output_dir="$project_root/dist"
port="${AETHER_WEB_PORT:-8080}"

if [[ "${1:-}" == "--port" ]]; then
  port="${2:-}"
fi

watch_fingerprint() {
  find "$project_root" \
    \( -path "$project_root/.git" -o -path "$project_root/dist" -o -path "$project_root/target" -o -path "$project_root/node_modules" \) -prune -o \
    -type f \( \
      -path "$project_root/apps/*" -o -path "$project_root/crates/*" -o -path "$project_root/games/*" -o \
      -path "$project_root/web/*" -o -path "$project_root/assets/*" -o \
      -path "$project_root/scripts/build-web.sh" -o -name Cargo.toml -o -name Cargo.lock \
    \) -print0 \
    | sort -z | xargs -0 -r stat --format='%Y:%n' | sha256sum
}

build() {
  "$project_root/scripts/build-web.sh"
  touch "$output_dir/.aether-reload"
  echo "[$(date '+%H:%M:%S')] Web build complete; refreshing connected browsers."
}

build
python3 "$project_root/scripts/web-dev-server.py" --directory "$output_dir" --port "$port" &
server_pid=$!

cleanup() {
  kill "$server_pid" 2>/dev/null || true
}
trap cleanup EXIT
trap 'exit 0' INT TERM

echo "Watching web sources at http://localhost:$port/"
fingerprint="$(watch_fingerprint)"

while true; do
  sleep 1
  next_fingerprint="$(watch_fingerprint)"
  if [[ "$next_fingerprint" != "$fingerprint" ]]; then
    echo "Source change detected; rebuilding web bundle..."
    if build; then
      fingerprint="$next_fingerprint"
    else
      echo "Web build failed; waiting for another source change." >&2
      fingerprint="$next_fingerprint"
    fi
  fi
done
