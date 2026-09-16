#!/usr/bin/env bash
set -euo pipefail

tool_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
runtime_dir="$tool_dir/.runtime"
comfy_dir="$runtime_dir/ComfyUI"

# shellcheck source=pins.env
source "$tool_dir/pins.env"
mkdir -p "$runtime_dir"

clone_at_revision() {
  local repository="$1"
  local revision="$2"
  local destination="$3"

  if [[ ! -d "$destination/.git" ]]; then
    git clone --filter=blob:none --no-checkout "$repository" "$destination"
  fi
  git -C "$destination" fetch --depth 1 origin "$revision"
  git -C "$destination" checkout --detach "$revision"
}

clone_at_revision "$COMFYUI_REPOSITORY" "$COMFYUI_REVISION" "$comfy_dir"
clone_at_revision \
  "$TEXTURE_ALCHEMY_REPOSITORY" \
  "$TEXTURE_ALCHEMY_REVISION" \
  "$comfy_dir/custom_nodes/ComfyUI-TextureAlchemy"

if [[ ! -x "$runtime_dir/venv/bin/python" ]]; then
  uv venv --python "$PYTHON_VERSION" "$runtime_dir/venv"
fi

torch_index_url="${COMFYUI_TORCH_INDEX_URL:-https://download.pytorch.org/whl/cpu}"
# CPU wheels are the space-conscious default. GPU workstations can point this at
# the appropriate official PyTorch CUDA/ROCm wheel index before setup.
uv pip install --python "$runtime_dir/venv/bin/python" \
  --index-url "$torch_index_url" \
  torch torchvision torchaudio
uv pip install --python "$runtime_dir/venv/bin/python" \
  -r "$comfy_dir/requirements.txt"

# Newer Kornia releases import the optional Rust accelerator eagerly. Its wheel
# can require CPU instructions unavailable in CI/dev containers. The supported
# 0.7.1 is pure PyTorch and covers both ComfyUI and this workflow.
uv pip install --python "$runtime_dir/venv/bin/python" "kornia==0.7.1"
uv pip uninstall --python "$runtime_dir/venv/bin/python" kornia-rs || true

printf 'ComfyUI and TextureAlchemy are ready at %s\n' "$comfy_dir"
