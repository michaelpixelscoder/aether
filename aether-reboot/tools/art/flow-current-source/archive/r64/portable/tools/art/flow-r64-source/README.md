# R64 — fresh coherent native flow asset, isolated first preview

**Original art and CPU pipeline ready. Native visual/runtime binding/performance unproved; no promotion.** This disjoint package on D preserves R53/R61/R62 shader baselines and the rejected R62 screenshot/evidence without touching the frozen R62 package. No world stage is duplicated and no GPU, Git or canonical Rust action is performed by this author.

## Asset strategy and exact provenance

The new original `source/world-flow-current-native.png` has native SHA`eaf534efc2a2e30632fcfee7f3d5b731efe77dcd19f0849216e6c74c662497c3`. It is genuinely new ImageGen art: three unequal coherent left-to-right cyan-blue crests, broad deep navy intervals and restrained violet forks. The PNG was inspected and copied byte-exact; no retouch, mask derivation, tint, alpha remapping, resize or false normal/height map is applied. Native transparency remains exact.

`source/prompt.txt` preserves the root's saved prompt including its added final LF. `source/prompt-invocation.txt` separately preserves the actual tool string without that one added LF; their hashes are independent in `native-r64.json`. Root reported the exact built-in image_gen.imagegen invocation: transparent_background=true, both reference arguments omitted, approximately3 October2026 10:35UTC,53.7s. The original generated path is provenance only; reconstruction never reads outside the portable package.

R62's narrowed dense network became broken cyan/violet blobs in the real engine and its foreground remained faint. It was rejected for promotion. R64 changes the source strategy before any shader knob search: the fresh native network is coherent itself. Shader is the frozen R62 optical model with only the necessary central crop.42+.29 on both broad routes and thermal, plus labels. Color, gain, coverage thresholds, phase transport, derivatives and core remain unchanged for this first controlled preview.

## Separate current asset: actual cascade conservation

New runtime texture is `assets/textures/world-flow-current-r64.ktx2`; historical `assets/textures/world-flow.ktx2` remains exactlySHA`8ecd62eb7df01a02d9864521e63b1c29a128ff18a267f369a1d7d96cd73b4973`. Root must bind the new handle only to material settings.x<.5. Waterfalls/pools must retain the old handle. Using the new KTX at the old filename would change waterfalls even with their shader branch unchanged, so this package explicitly rejects that architecture.

Root prepared a diagnostic `--current-texture` override, reported renderer binarySHA`b46d00534c3481b0a54099e2cd9b310c57f8db86857271c01656b2c11e4822b9`. This report is not evidence of a completed native render or binding capture. Use final canonical R59+R60 world/sky/weather for the preview; replace only the fixture shader and supply the new current-only texture path. No old whole-world manifest transplant is involved.

## Encoder contract, generic native dimensions

The encoder accepts the authenticated original RGBA8 dimensions from its metadata and PNG. It retains the strict8-bit/noninterlaced/true RGBA contract, source SHA and production minimum64px per axis; it does not silently invent alpha for RGB or accept a small substitute. It never resizes level0. Every mip uses linear-sRGB RGB multiplied by actual alpha, complete box-area averaging of both associated RGB and alpha, unpremultiplication, sRGB encoding and nearest-byte quantization. There is no alpha coverage correction, global opacity scaling, sharpening or recolor.

The current original is1254², producing11 floor-halved levels through1²,8,384,072 texel bytes and8,384,632 KTX bytes. Native level0 RGBA SHA`9bc3903ea5167a450682dd0cb848436ef21cc0b584ed74fb15eb95ce0369c18d` is byte-exact. Vulkan43/standard RGBA8sRGB, straight alpha and the DFD linear-alpha qualifier are verified independently. Both old and new textures retained together require16,768,144 raw texel bytes before runtime overhead; actual residency/FPS remains unmeasured.

The algorithm is adapted from the exact historical R53 encoder, archived unchanged with its proof/gate/native PNG. Fixed1254 assumptions are removed from the encoder, not replaced by weaker validation. Rectangular65×97 and64×128 textures, single-axis7×1/1×7 chains, last odd-edge contribution, constant RGBA, transparent-color poison and linear-light black/white tests guard full-footprint behavior.

## Rebuild and verify, no GPU

```text
python tools/art/flow-r64-source/record_source.py
python tools/art/flow-r64-source/build_mips.py
python tools/art/flow-r64-source/build_shader.py
node tools/art/flow-r64-source/verify.mjs
python tools/art/flow-r64-source/test_portability.py
```

The independent JS gate decodes the native PNG with CRC checks, proves level0 byte identity and actual dimensions/count/DFD/ranges/per-mip payload hashes, checks all historical waterfall/pool/haze shader branches and sampler, and rejects20 explicit corruptions. Source/generator hashes remain honest. Reconstruction outside the project on D produces identical KTX/shader bytes and reruns these checks; source directories and temporary cleanup targets are absolute-path verified. No native screenshot or performance conclusion is inferred from CPU success.

## Pending native decision

Check the coherent large arcs and foreground path at the concept33 camera, cyan/violet hierarchy against white clouds, continuous2–8px crests with dark gaps, absence of gray sheets,15–25px spiral eye, temporal/LOD/grazing stability and all unchanged cascades. R62's mask may still fragment dim portions despite the new image; this first native preview must reveal that before any further parameter change. The source itself includes soft edge transparency/vignette despite the prompt's request for edge usability, so tiling/seam behavior also needs close inspection. Historical gates, review and thresholds are preserved and no fidelity score is invented.
