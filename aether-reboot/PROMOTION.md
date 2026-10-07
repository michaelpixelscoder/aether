# R53 candidate — original 8K linear sky and source-matched IBL

This directory is an isolated promotion candidate. No canonical asset,
runtime Rust/WGSL, web bundle or sibling project has been changed. The parent
authorized an original 8192×4096 panorama to evaluate angular sharpness in
the engine, followed by one original 1024×512 authored IBL. Acceptance still
depends on native appearance and measured browser memory/performance.

The 8K output is RGBA16Float in uncompressed KTX2 (Vulkan format 97), with
one level, Rec.709 linear radiance, `rd` orientation, Repeat U and Clamp V.
It occupies exactly 268,435,456 bytes (256 MiB) of GPU texel storage. This is
192 MiB more than the previous 4096×2048 RGBA16F panorama. File delivery,
upload and CPU transient memory are additional costs to measure, not claimed
to be covered by that GPU figure. No alternative packed format was introduced.

## Authoring and source identity

R50 supplied the one actual .75-degree World emitter, its low direction and
the accepted continuous cloud fields. R52 removed the artificial bright
horizon band and added 223 affine placements in 12 continuously merged thin
fields. R53 changes only the neutral finite-air HG phase from .72 to .90.
The approved source therefore has 98 visible fields built from 4,013
placements of ten original density sculptures. The broad blue pole field,
actual emitter, cloud materials and the two air densities stay fixed.

The isolated R53 hero demonstrated actual near-source scattering and deeper
blue away from the source. Its opening remained cream/white and soft under
AgX; the desired golden opening is **not claimed solved**. The original hero,
R52 comparison, quantified HG prediction and CPU density-column diagnostic
remain in the candidate folders. Their manifests and source records are
also retained in `tools/art/sky-source/history`.

Before any final render, the packaged Blender file was saved at 8192×4096.
The complete descriptor comparison permits exactly two changes from the
approved source: `render.resolution_x` 4096→8192 and `resolution_y` 2048→4096.
All geometry, volume hashes, illumination and exposure agree. The proof is
`docs/evidence/sky-r53-pre-render-package.json`. There is no resize, upscale,
retouch, painted solar disk or camera-specific glow.

## Reproduction

Use Blender 5.2 with the paths below, relative to this package. The scene uses
only relative references inside `tools/art/sky-source`; no workstation path
is required by an active script. Launch background Blender with at most four
CPU threads and coordinate GPU ownership before `--final` or `--ibl`.

```
blender -b tools/art/sky-source/sky-world.blend --threads 4 --python tools/art/build_sky.py -- --verify-scene
blender -b tools/art/sky-source/sky-world.blend --threads 4 --python tools/art/build_sky.py -- --rebuild-volumes
blender -b tools/art/sky-source/sky-world.blend --threads 4 --python tools/art/build_sky.py -- --final
blender -b tools/art/sky-source/sky-world.blend --threads 4 --python tools/art/build_sky.py -- --ibl
node tools/art/verify-sky.mjs --self-test
node tools/art/verify-sky.mjs
```

`--rebuild-volumes` reproduces the bounded continuous VDB fields from the
recorded native sculptures and affine recipe. It only replays the saved VDB
serialization UUID; density and topology bytes must compare exactly.
`--final` renders original 8192×4096 / 128 maximum samples with RGB-only
denoising. It exports original Raw, zero-exposure RGBE radiance, then makes a
native-dimension linear unit conversion into RGBA16F. The power-of-two
encoding is chosen from the measured original radiance maximum to remain
finite in fp16; the reference artistic exposure remains -0.90 EV.

`--ibl` independently renders original 1024×512 / 128 maximum samples from
the same scene. Only the direct camera visibility of the actual disk changes.
Its light on every VDB and the finite air remain active. The validator compares
the full two scene descriptors and permits only resolution and that camera
disk flag to differ. The parent owns offline cubemap filtering and runtime
IBL integration.

The validator checks all source/input/evidence hashes, exact recipe rebuild,
KTX2 sections and DFD, every fp16 sample, original Radiance scanlines, seam
continuity and the full dimension chain: source scene, render record, original
HDR and KTX2. It accepts explicit original 4096×2048 or 8192×4096; it rejects
resampling, relabeled source dimensions and sub-100% original rendering.
`output_needs_render=true` remains a hard rejection.

## Integration constants

The actual game sun direction remains
`[-0.8535626531, -0.0509735756, -0.5184906125]`, with elevation
`-2.92183698°` and gallery azimuth `-121.276293°`. Its color is linear
`[1, .62, .32]`. The sole actual source has .75° diameter and normal red-channel
irradiance 22; engine key calibration and the already reviewed cloud bounce
remain the parent's responsibility.

Use the final manifest's measured `encoding_scale` and
`recommended_runtime_multiplier`, where gain equals `2^-0.90 / encoding_scale`.
The unencoded authored IBL reference gain is `2^-0.90 = .5358867312681466`
before the separately documented engine photometric scale. The raw RGBE IBL
does not inherit the panorama's fp16 encoding scale.

The existing panorama orientation remains `U = fract(.75 - mesh_uv.x)`,
`V = mesh_uv.y`, linear texture, Repeat U / ClampToEdge V and `RENDER_WORLD`.
No Rust change is part of this package. Final render/audit hashes, measured
range and exact gain are appended only after the original outputs exist.

## Completed original outputs

The original panorama finished in 403.12 seconds at 8192×4096 / 128 maximum
samples. Original HDR radiance range is [.048828125, 194560]. Encoding .25
produces fp16 RGB range [.01220703125, 48640], with alpha exactly one. The
exact runtime multiplier is **2.1435469250725863**, or Rust f32 literal
**`2.143_546_9`**. This is unchanged from the R50 encoding convention.

The original IBL finished in 12.48 seconds at 1024×512 / 128 maximum samples,
with radiance range [.046875, 11.625]. It is unencoded Raw RGBE; retain the
reference gain **.5358867312681466** before the parent's calibrated IBL scale.
Neither output received a view transform or exposure during export.

| Artifact | SHA-256 |
| --- | --- |
| Packaged original 8K `.blend` | `4196ff61f70037e5fb373ccdd840c52ff714fc82e6b03ae243b1cea65571acea` |
| Original visible RGBE HDR | `e588df650412ae7fe6ace40ecbf78367d8583996409e059cf9fae3e653ff2b99` |
| Native RGBA16F KTX2 | `a26bfa6900f6e46cab19738b01027dd7f1e33c6a682787bac3ab0c0bb04193fb` |
| Original camera-disk-free IBL HDR | `789a304b0f403e3dcee09268b0883b87d4266bf83c621e117fa46affa55fb075` |

The KTX2 file is 268,435,736 bytes including 280 bytes of container data.
The full main scene descriptor hash is
`6c214eef069d7008cdadd466e1f569ed59bfe4ab2f6cd58a3745bc8b4cd3cd4b`.
The IBL descriptor hash is
`c3c0db5235793fe6459f615588458471d6d58cda04e6e71c28fb2ae553ad59ac`.
Their only differences are the authorized original resolution and direct
camera disk flag. GPU ownership was released after both render processes
exited; no further image render is planned in this candidate.

## Final validation

The full candidate validator passes **189 SHA records**, including all used
sculptures/fields, the packaged source, render controller, original outputs
and recorded provenance. Every fp16 sample is finite and nonnegative, alpha
is one, and the black-pixel fraction is zero. The exposure-compressed seam
mean is .00184811 versus .00126839 for ordinary adjacent columns, within the
existing acceptance limits. Both poles and the whole-sphere dimensions are
also checked; the image has not been warped to repair a seam.

The 98 derived VDB fields rebuilt from the 4,013 recorded placements in 48.32
seconds with **all original SHA-256 values identical**. A fresh Blender reopen
validates the exact 8K descriptor and every relative path. The new validator's
self-tests include accepted original4K/8K and rejected relabeled/rescaled
provenance. It also validates the unchanged R50 original4K package, with its
report saved here as `docs/evidence/verify-legacy4k.json`.

The standard `node tools/art/verify-sky.mjs` invocation writes
`docs/evidence/sky-assets.json`, retaining the contract consumed by
`verify-open-world.mjs`. This verifies sky-call compatibility; it is not a
claim that the entire unrelated open-world asset suite was rerun.

Legacy R50 equivalence evidence was moved into `history` and remains hashed;
this candidate is rendered directly from its packaged 8K source and does not
reuse that old equivalence claim. `PROMOTION.json` is the final portable file
inventory. Generated Python caches and `.dream-loop` rebuild scratch are
excluded from it and are not promotion inputs. Canonical and prior candidate
immutability is recorded in `docs/evidence/sky-r53-frozen-source-audit.json`.
