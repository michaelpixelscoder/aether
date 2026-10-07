# World flow material

The final medium-violet shader was accepted in the parent native capture `review/world-r53-violet.png`. The earlier dark R53 shader and the stronger whitening variant are preserved in `history/`, with their native review frames. No new geometry, physics, route, clock or Rust behavior is part of this asset package.

Runtime assets:
- `assets/shaders/world-flow.wgsl`
- `assets/textures/world-flow.ktx2`

The native ImageGen RGBA and exact prompt are `source/world-flow-native.png` and `source/world-flow-prompt.txt`. Native pixels remain untouched. The texture is1254²; KTX2 level0 is byte-exact decoded native RGBA. Eleven uncompressed Rgba8UnormSrgb levels use8,384,072 GPU bytes. Derived mips average linear RGB weighted by alpha and average alpha, then unpremultiply and encode sRGB. NPOT floor-halving; no alpha coverage rescale. The authored PNG is retained only with the author sources, not loaded at runtime.

Rebuild from the workspace root:
```text
python tools/art/flow-source/build_mips.py
python tools/art/flow-source/build_shader.py
python tools/art/flow-source/review_material.py
node tools/art/flow-source/verify.mjs
```
Python requires NumPy. Numerical libraries are limited to4threads. The no-argument validator derives the workspace from its own location; it also accepts `--root <workspace>`. No absolute working-directory dependency, engine, GPU or network is needed.

The source shader is reconstructed from the pinned reviewed medium and one explicit color-only patch. Current crests use positive RGB energy, including blue, at gain14. Cascade gain is12. Low body opacity avoids the grey absorption sheet of the rejected R53. A native red/green ratio selects a few authored portions of existing current crests for violet tint. Current alpha, UVs, ridge widths, body RGB and the complete cascade branch remain exact to the reviewed medium. CPU probes find tint on16.7–18.6% of visible ridge samples at usual probe sizes. This is a surface observation, not an art score.

Renderer contract: bindings0/1 are existing color/settings; binding2 is sRGB2D texture; binding3 is Repeat/Linear mag/min/mip. AlphaMode::Blend expects straight RGB+alpha. Periodic two-phase interpolation temporarily associates color, then unassociates it. UV gradients are evaluated before fract. Two texture fetches per fluid fragment. Simulation time remains settings.w×settings.y; vertex alpha, near-current fade, pools and atmosphere are unchanged. No vertex-red use.

Validation independently decodes the PNG, checks all KTX2 DFD/index/payloads, pins the final runtime shader, verifies actual author-source hashes and the exact allowed hue patch, preserves clock/pool/haze contracts, and tests the Naga reserved-word guard with the formerly rejected `patch` identifier. That guard is not a full shader compiler; native parent capture supplies the actual compilation evidence. Source/proof hashes and native review hashes are recorded in `source/provenance.json`.

The relocation test rebuilds native-derived KTX and final shader from a copied package, reruns CPU proof and the no-argument validator, and checks byte identity. Result: `proofs/relocation.json`. Archived JSON reports describe historical candidates; the production verifier targets only the final runtime shader and current author sources. No canonical file was changed by the author while preparing this package.
