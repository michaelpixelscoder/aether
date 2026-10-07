# R55 — local cloud attribution and directional reflected light prototype

Everything in this folder is isolated. R53's original sky, authoring scene,
manifest and IBL are unchanged. No canonical Rust, WGSL, density field, bank
definition or image has been modified. No GPU render/capture has been run by
this task. The prototype shader is `assets/shaders/clouds.wgsl`.

## Attribution from the captured inputs

The captured frame is `../world-r54-sky.png`, SHA
`4b7b5e96633aec66f397fe4a683afa32e1037af60f90d7ebe214a02e899b13d4`.
The diagnostic verifies the exact cloud shader, bank JSON, six 3D caches and
original sky against `world-r54-sky.evidence.json`. It samples the native 8K
RGBA16F panorama at the unchanged camera rays, reconstructs the runtime
64³ periodic noise and volumetric integration, then applies Bevy's local
ACES implementation for numeric comparison. It generates no raster image.

The two prominent dark rounded lobes in x0–255,y0–105 are **the WEST runtime
bank**. Other high banks do not contribute to the audited cone. On a coarse
112-ray grid, 56 rays reach the shader's opacity cutoff, 33 are partly
attenuated and 23 are clear. These numbers exclude opaque island depth and
are an attribution of cloud support, not a pixel-perfect coverage mask.

| Pixel | Panorama alone, CPU ACES RGB | With runtime bank, CPU RGB | Captured 3×3 RGB | WEST transmission |
| --- | --- | --- | --- | ---: |
| (175,20) | 211,201,193 | 69,87,117 | 77,92,118 | .0119 |
| (200,30) | 241,233,223 | 66,83,113 | 74,88,114 | .0104 |
| (45,30) | 118,111,116 | 75,91,119 | 82,95,119 | .0116 |
| (15,100) | 96,92,99 | 96,92,99 | 99,94,100 | 1 |

The final row matters: the extreme left/lower part of the requested rectangle
also contains a genuinely shaded authoring cloud in the panorama. It would
be incorrect to attribute that entire rectangle to local clouds. At (250,100)
an opaque island invalidates a sky-only comparison, and nearby flow/bloom
also affect some samples; neither is silently treated as cloud error.

The CPU reconstruction uses midpoint jitter and scalar f64 arithmetic rather
than GPU f32. It does not reconstruct opaque depth, TAA or bloom. Perturbing
all sampled noise channels by ±2/255 leaves the strong core attribution
unchanged, but this sensitivity test is not advertised as a formal error
bound. The matched native core colors provide an independent check.

The parent's later RGB9E5 experiment replaced the review-stage sky. The
prototype CPU comparison therefore explicitly reads the still-frozen R53
RGBA16F file and requires its original R54 SHA. No hash check or acceptance
tolerance was relaxed. `diagnosis.json` preserves the first baseline study;
`prototype-comparison.json` records the subsequent explicit source location.

## Why the local lobes are so dark

At (200,30), the opacity-weighted macro-normal dot solar direction is -.869
and the four-tap solar optical depth is approximately 9.68. The direct
contribution integrates to only (.0143,.0121,.0093), while the fixed ambient
contributes (.0687,.1232,.2328). Consequently the visible core is almost
entirely the hard-coded blue ambient. The direct lobe also retains a surface
normal gate with minimum .20; it is not a full volume scattering model.

The R54 cloud shader receives the actual low solar direction, but neither
the source-matched IBL nor the upper cloud-bounce approximation applied to
objects. Its direct chromaticity (1,.85,.65) also differs from the current
actual source (1,.62,.32). Raising a global ambient scalar would flatten the
clouds and leave these separate lighting systems unresolved.

## Isolated structural lighting change

The prototype retains the entire existing direct-light/ambient calculation
for a controlled comparison. It adds the same documented reflected-light
direction as the object bounce: solar azimuth, elevation 42°, relative
strength .35, linear warm chromaticity (1,.62,.32). The existing cloud red
radiance scale3 is retained; this is relative scene calibration, not a claim
that cloud shader units and lux are already identical.

The added incoming lobe has its **own four density shadow samples** through
the unchanged volume. It uses the existing phase/powder response and two-scale
attenuation approximation. There is no extra Lambert normal gate for these
participating particles. Thus an exposed upper fold receives warm light
while a blocked fold keeps its cool shadow. This is a directional reflected
illumination approximation, not a second stellar disk, not a constant fill,
and not an exact multiple-scattering solution.

The material binding layout, 128 view steps, early-exit threshold, camera
depth clipping, all density/noise functions, macro caches, masks and cave
floor continuity are byte-identical to the captured baseline. The change
applies consistently to all six banks; there is no WEST-only screen mask.

## CPU estimate and regional checks

Every before/after ray retains **exactly the same transmission**, asserted
in the comparison program. The following are displayed ACES estimates without
opaque geometry, post-process bloom or TAA, not fabricated screenshots.

| Sample | Current RGB | Prototype RGB |
| --- | --- | --- |
| WEST upper exposed fold (175,20) | 69,87,117 | 188,171,161 |
| WEST front core (200,30) | 66,83,113 | 113,111,125 |
| WEST sheltered fold (225,55) | 71,84,111 | 83,91,114 |
| WEST left core (45,30) | 75,91,119 | 92,101,123 |
| Regional exposed fold (1120,380) | 73,91,121 | 196,178,168 |
| Regional shaded column (600,580) | 46,70,105 | 98,100,119 |
| Regional lower fold (920,830) | 92,103,124 | 136,131,138 |

Both HOLLOW and UNDERFORGE are explicitly evaluated. At (1120,380), the
added red contribution is .688 for HOLLOW and .413 for UNDERFORGE. At
(1170,570), it is only .093 for HOLLOW versus .327 for UNDERFORGE, demonstrating
distinct path-dependent illumination. The image composition remains governed
by their unchanged transmittance. Some regional exposed folds brighten a lot;
the native review must check that these remain white/gold articulations rather
than a broadly beige cloud deck. Dark interior/cave protection is also a
required runtime check, not inferred from these exterior rays.

There is a real performance cost: shading now evaluates eight shadow-density
samples instead of four per shaded march step. Density steps, caching and
early-outs are retained; no claim of unchanged frame time is made. Native/Web
profiling should measure the incremental cost after visual acceptance. A
later directional visibility cache could remove repeated work, but no such
architecture or approximation has been added to this trial.

## Validation and review handoff

`node build_prototype.mjs` generates only the isolated shader and CPU study;
`node compare_prototype.mjs` writes the numeric comparison. Naga parsing and
validation pass using local import stubs (`check_wgsl.rs` and
`prepare_shader_check.mjs`). Actual Bevy import/binding/pipeline compilation
still requires the parent's isolated native capture. A first attempt to link
against release LTO libraries failed before shader validation; the check
completed successfully using the existing debug Naga library instead.

The next step is one native frame with this shader over the unchanged R53
sky and .35 object bounce. Inspect WEST, both regional banks and the whole
frame against reference33; retain or reject based on that real result.
No promotion or new numerical tolerance is implied by the CPU estimate.
