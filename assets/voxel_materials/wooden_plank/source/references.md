# Wooden plank generation resources

- [ComfyUI](https://github.com/Comfy-Org/ComfyUI), GPL-3.0. Used as the pinned
  local workflow runtime and HTTP API.
- [ComfyUI-TextureAlchemy](https://github.com/amtarr/ComfyUI-TextureAlchemy),
  Apache-2.0. Pinned nodes used here: Height to Normal, AO Approximator,
  Channel Packer, Seamless Tiling, and Texture Tiler.
- Project workflow research:
  `design/voxel-shading/research/generative-pbr-texture-workflow.md`.

No third-party photographic reference or diffusion checkpoint is embedded in
this first test. `albedo_candidate.png` is the accepted project-generated source
candidate. Its SHA-256 is refreshed in `inputs/inputs.json` on each build.
