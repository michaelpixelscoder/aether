#!/usr/bin/env bash
set -euo pipefail

output_dir="${1:-target/voxel-render-gallery}"
if [[ $# -gt 0 ]]; then
  shift
fi
command=(cargo run -p aether_voxel_render_tests -- --output "$output_dir" "$@")

if [[ -z "${DISPLAY:-}" ]] && command -v xvfb-run >/dev/null 2>&1; then
  xvfb-run -a "${command[@]}"
else
  "${command[@]}"
fi

printf 'Open %s/index.html\n' "$output_dir"
