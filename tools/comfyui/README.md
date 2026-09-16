# Project-local ComfyUI

This directory pins the authoring runtime while keeping ComfyUI, model weights,
caches, and disposable output out of Git. Selected workflows, prompts, source
images, masks, maps, and validation previews are stored with each material under
`assets/voxel_materials/<material>/`.

## Install and open

```bash
tools/comfyui/setup.sh
tools/comfyui/start.sh
```

Open <http://127.0.0.1:8188>. The pinned revisions are in `pins.env`.
The installer defaults to compact CPU-only PyTorch wheels. On a GPU workstation,
set `COMFYUI_TORCH_INDEX_URL` to the appropriate official PyTorch wheel index
before setup. Set `COMFYUI_FORCE_CPU=1` when starting to force CPU execution.

## Generate technical maps for an asset

The general command is:

```bash
scripts/generate_tech_maps assets/voxel_materials/wooden_plank
```

The command reads optional `tech_maps.json` path overrides and
`texture_prompt.md`, runs the asset's preparation hook, starts ComfyUI if needed,
submits `source/workflow.api.json`, writes maps to `textures/`, runs the asset
validator, and publishes configured runtime aliases. The prompt text and hash are
included in the ComfyUI request and `comfyui-run.json`.

Without configuration, these paths are assumed:

```text
<asset>/texture_prompt.md          # optional
<asset>/source/prepare_inputs.py
<asset>/source/workflow.api.json
<asset>/source/inputs/
<asset>/textures/
<asset>/validate.py                # optional
```

No diffusion checkpoint is required for the deterministic map-coherence pass.
Future albedo-generation workflows can add model references without changing
the asset layout.
