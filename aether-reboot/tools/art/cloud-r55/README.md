# R55 directional reflected cloud light

This package adds a spatially shadowed warm upper light lobe to all six runtime
cloud banks. It preserves the accepted cloud fields, composition, original sky,
128-step integration, depth clipping, cache bindings and cave floor. The lobe
matches the documented object cloud-bounce approximation: solar azimuth,
42-degree elevation, relative strength .35 and linear color (1,.62,.32).
It is reflected incoming illumination, not a second sun or a painted glow.

The old shader's direct and ambient terms are retained. The additional lobe
uses its own four density-shadow samples, phase and powder response. It does
not receive a surface Lambert normal gate. The existing two-exponential
attenuation approximation remains an approximation, not an exact solution for
multiple scattering. Cloud radiance retains its legacy red-channel scale 3;
the .35 ratio does not assert an identity between shader radiance and lux.

`assets/shaders/clouds.wgsl` is the reviewed output. Its source is the immutable
`archive/captured-clouds-r54.wgsl` plus `../build_cloud_lighting.mjs`.
The generator neither reads a live staging directory nor changes density.
Its output SHA is
`ffcc09fbaea8702ddcab1938c3ee1e2c3c8c5431a765a1c00002b2f7ef9119b6`.

## Reproduction and validation

From the repository/package root:

```powershell
node tools/art/build_cloud_lighting.mjs
node tools/art/verify-cloud-lighting.mjs
```

To recompute the R55 CPU comparison, set `BLENDER_EXE` to a local Blender
executable with bundled NumPy, then run:

```powershell
$env:OMP_NUM_THREADS = '4'
$env:OPENBLAS_NUM_THREADS = '4'
$env:MKL_NUM_THREADS = '4'
& $env:BLENDER_EXE --background --factory-startup --threads 4 --python tools/art/compare_cloud_light_rays_cpu.py
node tools/art/verify-cloud-lighting.mjs
node tools/art/cloud-r55/test-gate.mjs $env:CLOUD_VALIDATION_SCRATCH
```

`CLOUD_VALIDATION_SCRATCH` must name a separate scratch directory. Mutation
tests copy the necessary files there and never corrupt the source package.
They test stale shader/source proofs, a missing bounce, old lighting, changed
cameras, omitted runtime cadence, altered opacity, rewritten R47 evidence and
loosened thresholds. No GPU is used by these operations.

The six caches and their generator are unchanged and included so the package
is independently reviewable. Their original CPU rebuild and R47 validation
commands remain in `../cloud-r47/README.md`. Do not regenerate or relabel R47
proofs as R55 results. The new build integration is an **additional**
`node tools/art/verify-cloud-lighting.mjs` after the existing cloud-cache gate.
The historical cache gate and all of its source files stay byte-identical.

## Separate historical and current evidence

R47 retains its own contract, image proof, scalar/derivative audits, sources
and failed historical .035 regional pointwise gate. Its artistic acceptance
used an explicitly reviewed .05 regional p99 limit with image/derivative
guards; this package makes no new exception and does not rewrite that history.

R55 writes `docs/evidence/cloud-r55-ray-comparison.json` and
`docs/evidence/cloud-r55-lighting-fidelity.json`. The same four views cover
HOLLOW and UNDERFORGE at oblique and grazing angles, at 240×150, 128 steps,
identical fixed-noise values, alpha categories, orthographic rays and display
transform. The limits remain opaque display RMSE < .025 and edge alpha p99
< .08. The validation additionally requires opacity errors to match R47
exactly, since illumination must not change extinction.

Illumination is explicitly updated to the actual low solar direction and
the R55 directional bounce, including the shader's alternating lateral
shadow-ray offsets. The four views run twice: the historical CPU four-step
lighting cadence, and the actual shader's two-step cadence. This is eight
comparisons, not eight newly chosen cameras. Both sets must pass the same
limits. Analytic density/normal, cached density/normal and cached density
with analytic normals separate scalar and normal errors.

CPU images and raw float arrays are in `review/`. They are direct diagnostic
volume integrations, not engine screenshots or edited concept assets.
Their fixed noise, midpoint samples and orthographic camera deliberately
preserve the R47 field test. They do not reproduce live advection, opaque
scene depth, temporal reconstruction, bloom or the engine's ACES transform;
therefore these comparisons establish cache fidelity under new illumination,
not pixel identity with native Dawn or cave renders.

## Native review and remaining limits

The two unmodified native captures are archived as
`world-r55-cloud-bounce.png` and `world-r55-cloud-hollow.png`. The parent reviewed
better exposed folds with cool cores retained, and no obvious cave lighting
leak. The top-left WEST lobes improve because actual shadow paths admit more
upper light, not because their density or silhouette was replaced. Some of
the darkest extreme-left pixels originate in the original sky and remain.
The full target's broad golden opening is not claimed solved by this change.

Shadow-density samples per shaded view step rise from four to eight. Parent
native frame p95 was 14.99 ms versus 14.42 ms in R54, but other changes were
present, so this is not an isolated shader-cost measurement. Web performance
and final production acceptance are the parent's responsibility. No new GPU
render, canonical mutation or sky export was performed for this package.

The initial attribution reports are frozen under `archive/`; they retain
their original historical source descriptions. Those paths are explanatory
history and are not dependencies of the portable generator or validation.
