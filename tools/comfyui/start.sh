#!/usr/bin/env bash
set -euo pipefail

tool_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
runtime_dir="$tool_dir/.runtime"
comfy_dir="$runtime_dir/ComfyUI"

if [[ ! -x "$runtime_dir/venv/bin/python" ]]; then
  printf 'ComfyUI is not installed. Run %s/setup.sh first.\n' "$tool_dir" >&2
  exit 1
fi

cd "$comfy_dir"
device_arguments=()
if [[ "${COMFYUI_FORCE_CPU:-0}" == "1" ]] || \
  ! "$runtime_dir/venv/bin/python" -c 'import torch; raise SystemExit(0 if torch.cuda.is_available() else 1)' 2>/dev/null; then
  device_arguments+=(--cpu)
fi
exec "$runtime_dir/venv/bin/python" main.py \
  "${device_arguments[@]}" \
  --listen 127.0.0.1 \
  --port "${COMFYUI_PORT:-8188}" \
  --disable-auto-launch
