# Generative PBR texture workflow research

Date: 2026-09-13

## Outcome

Use a hybrid, reproducible pipeline:

1. **ComfyUI** orchestrates image generation and records the graph, prompt, model, seed, and inputs.
2. Generate or edit one high-quality, flat-lit **base-color concept**. Do not independently prompt every PBR map.
3. Make the material mathematically tileable, then apply the exact same spatial transformation to all derived maps.
4. Derive a canonical 16-bit height field, then derive the OpenGL normal and ambient-occlusion maps from that height field. Author or estimate roughness and metallic using material-aware rules.
5. Pack runtime channels for Bevy and validate both pixels and relit voxel renders.
6. Keep generated candidates and workflow metadata as source assets; publish only reviewed runtime maps.

This gives us AI-assisted exploration without making model output the authority for physical material relationships.

## Important terminology

- **Base color/albedo:** surface color with no baked directional light, cast shadow, or highlight.
- **Height/displacement:** one scalar description of surface elevation. Keep this as the canonical relief source.
- **Bump:** a renderer technique that can consume height. It is not a second independent source map.
- **Normal:** derived directional surface detail. Use OpenGL orientation for Bevy, or explicitly set `flip_normal_map_y` when importing DirectX normals.
- **Roughness:** microfacet scattering, not simply inverted brightness.
- **Metallic:** a material classification. Wood is 0; nail heads are 1. Avoid noisy gray metallic maps unless the material truly mixes dielectric and metal regions.
- **Reflection:** not a generic extra grayscale map in the default metallic/roughness workflow. Reflection is primarily controlled by base color, metallic, roughness, environment lighting, and Bevy’s reflectance/specular facilities.
- **AO:** indirect-light occlusion for small cavities. It must not reproduce the base-color image as a generic darkening layer.

For runtime compatibility, follow glTF/Bevy channel packing: AO in red, roughness in green, metallic in blue. Load normal, height, AO, roughness, and metallic data as linear rather than sRGB.

## Existing tools

| Project | License and maturity | Useful capability | Decision |
| --- | --- | --- | --- |
| [ComfyUI](https://github.com/Comfy-Org/ComfyUI) | GPL-3.0; large, active project | Versionable node graphs, local/API execution, models, reference conditioning, inpainting, upscaling, high-bit-depth output, workflow/seed metadata | **Use directly as optional orchestration**, outside the game/runtime dependency graph |
| [Dream Textures](https://github.com/carson-katri/dream-textures) | GPL-3.0; mature Blender add-on, but its public workflow targets older Blender/Stable Diffusion generations | Circular convolution for seamless axes, inpainting seams, local generation, AI upscaling | **Reference implementation**, especially circular-padding generation and seam inpainting; do not make Blender a mandatory build dependency |
| [ComfyUI-Seamless-Texture](https://github.com/vanillasoap/ComfyUI-Seamless-Texture) | README declares MIT; very young/small project | Delighting, offset/multiband/mirror tiling, one identical transform across a PBR set, tile preview, seam heatmap | **Pilot directly after code review and pinning**; otherwise port the small algorithms and tests with attribution |
| [ComfyUI TextureAlchemy](https://github.com/amtarr/ComfyUI-TextureAlchemy) | Apache-2.0 | AI depth inputs, height↔normal conversion, DX↔OpenGL conversion, AO/roughness/metallic processing, seamless operations, PBR preview | **Evaluate as the first map-processing prototype**; pin a reviewed revision because it is a custom-node dependency |
| [Material Maker](https://github.com/RodZill4/material-maker) | MIT; established project with thousands of commits and releases | Procedural PBR graph, 3D preview, albedo/metallic/roughness/emission/normal/depth authoring, customizable engine exports | **Use directly for artist correction and reference exports**; borrow its graph/export concepts for our manifest |
| [DeepBump](https://github.com/HugoTini/DeepBump) | GPL; established Blender tool and CLI | ML-generated normal from color, height from normal, curvature, upscaling | **Use as a comparison backend**, not the sole authority; it covers relief but not a full material set |
| [MatFuse](https://github.com/giuvecchio/matfuse) | MIT; official CVPR 2024 research implementation | Joint diffuse/normal/roughness/specular SVBRDF generation from text, image, palette, and sketch | **Research spike only**; coherent joint maps are promising, but inference requires at least 12 GB VRAM and tileability still needs validation |

### Why not adopt one tool as the complete pipeline?

- Image generators optimize appearance, not physical correctness.
- Single-image map extractors infer shape from color and can turn painted grain or highlights into false depth.
- Seam tools can remove edge discontinuities while introducing mirrored repetition or blurred structure.
- Joint material models improve map coherence but do not automatically match our stylized art direction, resolution, tile constraints, Bevy channel layout, or border semantics.
- Custom ComfyUI node repositories can be abandoned or compromised. Pin exact commits, inspect code, and keep the deterministic post-processing/validation path runnable without downloading arbitrary nodes.

## Proposed workflow

```text
material brief + references + mask/layout guides
                    │
                    ▼
        AI candidate generation/editing
          ComfyUI or approved image API
                    │
          4-8 base-color candidates
                    │
                    ▼
        flat-light/delight + art review
                    │
                    ▼
     seamless transform chosen and recorded
   circular generation / offset-inpaint / multiband
                    │
                    ▼
       canonical height + material masks
          │              │
          │              ├── roughness
          │              └── metallic
          ├── OpenGL normal
          └── AO
                    │
                    ▼
   identical wrap repair across every map + QA
                    │
                    ▼
 base_color.png + normal.png + height.png + orm.png
                    │
                    ▼
  Bevy relighting gallery at 1x / 2x / 4x / 8x repeat
                    │
                    ▼
             approve or iterate
```

### 1. Brief and structural guides

Every generation begins with a material brief, not only a prompt. It defines:

- physical material classes and masks;
- texel scale in voxel-body units;
- directional structure and allowed rotations;
- intended borders, fasteners, seams, damage, and which belong in separate overlays;
- target values or ranges for roughness, metallic, and relief amplitude;
- forbidden baked lighting, perspective, recognizable objects, text, and non-tileable edge features;
- reference provenance and license.

For structured materials such as wooden planks, provide a repeatable layout/mask guide to the image model. This keeps board count, seam locations, and nail masks stable across revisions.

### 2. Candidate generation

- Generate several square candidates at a fixed prompt revision and record each seed/model/service.
- Prefer flat, orthographic material capture language: no perspective, vignette, cast shadows, or directional highlight.
- Generate base color or a jointly modeled SVBRDF set. Never generate normal, roughness, metallic, and height as four unrelated text-to-image requests.
- Upscale before fine PBR extraction when the generation backend is low resolution, but validate that the upscaler uses wrap-aware tiles or overlap.

### 3. Perfect repetition

“Looks seamless once” is not the gate. A passing texture must:

- have matching opposite-edge values within a defined tolerance for every map;
- preserve first-derivative continuity across height and normal seams;
- show no visible cross in offset view;
- remain convincing in 2×2, 4×4, and 8×8 previews;
- avoid obvious mirrored motifs or periodic landmarks;
- use the same crop, offset, warp, resize, and seam mask for every PBR channel.

Preferred techniques by material:

- **Generation-time circular padding:** good default when the model/backend supports it.
- **Offset + masked inpaint:** best for structured materials because it repairs the actual seam without mirroring the whole image.
- **Multiband blending:** good deterministic fallback for noisy or moderately structured textures.
- **Mirror quad:** mathematically seamless but a last resort because symmetry can be obvious.

After any later edit to any map, rerun the entire seam and coherence gate.

### 4. Coherent PBR maps

Use a shared representation:

- The height map is the authority for relief.
- Normal is deterministically derived from wrap-aware height gradients.
- AO is derived from the same periodic height field and limited to crevices.
- Roughness starts from material-class masks plus bounded local variation; it is reviewed under moving highlights.
- Metallic comes from explicit material masks, not luminance heuristics.
- Optional specular/reflectance is added only for a measured artistic need and only after enabling the corresponding Bevy feature.
- The base color is checked for lighting information that conflicts with normal/roughness response.

If an AI model generates all maps jointly, treat them as proposals. Reconstruct height/normal consistency, clamp physically invalid values, and verify material masks before approval.

### 5. Project artifact contract

```text
assets/voxel_materials/<material-id>/
  material.ron                 # runtime parameters and stable texture references
  textures/
    base_color.png             # sRGB
    normal.png                 # linear, OpenGL convention
    orm.png                    # linear: R=AO, G=roughness, B=metallic
    height.png                 # 16-bit linear source; optional at runtime
  source/
    brief.md
    workflow.json              # ComfyUI/API graph with variable slots
    generation.json            # model, version, seed, prompt, inputs, hashes
    layout.png                 # optional structural guide/mask
    masks/
    candidates/
    pbr-unpacked/              # lossless working maps before ORM packing
  previews/
    tile-4x4.png
    channels.png
    relit-contact-sheet.png
  LICENSES.md
```

Large source/candidate images should use Git LFS. Runtime maps should have deterministic names; candidate filenames include a generation ID and must never be referenced by game code.

### 6. Bevy material target

The first PBR implementation should populate:

- `base_color_texture` from sRGB base color;
- `normal_map_texture` from a linear OpenGL normal map;
- `metallic_roughness_texture` and `occlusion_texture` with the same linear ORM image;
- scalar `perceptual_roughness` and `metallic` as non-destructive multipliers;
- repeat samplers on every map, with identical UV transforms.

Keep height/displacement in the source set even before runtime parallax or displacement exists. A future `ExtendedMaterial` can consume height without regenerating the material.

## Automated quality gates

| Gate | Automated check | Review artifact |
| --- | --- | --- |
| Completeness | Required files, dimensions, bit depth, manifest fields, and matching sizes | Channel grid |
| Exact tiling | Opposite-edge RMSE/max error for each map; periodic gradient check for height/normal | 4×4 and 8×8 tile previews plus seam heatmap |
| Map coherence | Normal reconstructed from height compared by angular error; AO correlation limited to cavities | Height/normal/AO diagnostic |
| PBR semantics | Metallic-class histogram and allowed masks; roughness bounds; flat-normal validity; normal length | False-color material-class view |
| Color space | Base color tagged sRGB; data maps loaded/exported linear | Manifest report |
| Repetition | Autocorrelation/feature repetition warning beyond the unavoidable tile period | Large tiled preview |
| Relighting | Render the same voxel fixtures under studio, grazing, warm, cool, and neutral lighting | Automated voxel render gallery |
| Provenance | Prompt, model/version, seed, input hashes/licenses, tool revisions | `generation.json` |

Numeric gates catch broken files, not artistic quality. A human must still approve scale, material identity, repeated motifs, false relief, highlight behavior, and the fit with *Aether Isles*.

## Iteration plan

### Experiment A — Wooden plank pipeline

1. Reproduce the current `wooden_plank.png` look from a checked-in brief and layout guide.
2. Produce 4-8 candidate base-color maps.
3. Compare generation-time circular padding with offset-inpainting and multiband repair.
4. Generate height/normal/AO using TextureAlchemy, DeepBump, and a deterministic wrap-aware baseline.
5. Author wood/metal masks so wood remains dielectric and nails remain metallic.
6. Export base color, OpenGL normal, 16-bit height, unpacked AO/roughness/metallic, and packed ORM.
7. Render all voxel fixtures under multiple lights.
8. Select one failure, correct it, and rerun every gate before adoption.

### Experiment B — Contrasting material

Repeat the same workflow with stone or metal plate. The pipeline is not considered generic until its parameters and maps work for a material whose structure and reflectance differ from wood.

## Recommendation

Start with **ComfyUI + reviewed/pinned TextureAlchemy and Seamless-Texture nodes**, while keeping **Material Maker** as the interactive correction/preview tool. Implement our own small command-line packer and validator because Bevy naming, ORM layout, wrap-aware map coherence, provenance, and voxel render-gallery integration are project-specific.

Run **MatFuse** only as a benchmark spike if suitable GPU capacity becomes available. It is the strongest researched reference for jointly coherent material generation, but it is too heavy and insufficiently tile-focused to define the first production workflow.

Do not copy third-party code into the repository until its exact revision, transitive model licenses, and redistribution obligations are recorded.

## Sources

- [ComfyUI repository and workflow/API features](https://github.com/Comfy-Org/ComfyUI)
- [Dream Textures seamless generation documentation](https://github.com/carson-katri/dream-textures/blob/main/docs/IMAGE_GENERATION.md)
- [ComfyUI-Seamless-Texture algorithms and PBR batch processing](https://github.com/vanillasoap/ComfyUI-Seamless-Texture)
- [ComfyUI TextureAlchemy PBR processing nodes](https://github.com/amtarr/ComfyUI-TextureAlchemy)
- [Material Maker repository](https://github.com/RodZill4/material-maker)
- [Material Maker PBR overview](https://github.com/RodZill4/material-maker/blob/master/material_maker/doc/intro.rst)
- [DeepBump](https://github.com/HugoTini/DeepBump)
- [MatFuse official implementation](https://github.com/giuvecchio/matfuse)
- [Bevy `StandardMaterial`](https://docs.rs/bevy/latest/bevy/pbr/struct.StandardMaterial.html)
- [glTF 2.0 metallic/roughness material specification](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html)
