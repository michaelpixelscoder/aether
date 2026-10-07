# Original 4K depth sky — portable source

The active proposal is an original 4096×2048 Cycles HDR, packaged as linear
RGBA16Float KTX2. No raster input, retouching, panorama warp or resizing is used.
The game samples Repeat U / ClampToEdge V with `U=fract(.75-u), V=v`.
Encoding scale is .5; runtime radiance gain is 1.0717734625362931
(`1.071_773_5_f32` proposed in Rust). GPU payload: 64 MiB. No 8K is claimed.

`sky-world.blend` contains 86 visible continuous density fields, one physical
sun, the original sky/air rig and full-sphere camera. Hidden authoring objects
were removed and paths remapped. This packaged scene has not been rerendered.
`render-equivalence.json` records identical render-relevant descriptors before
and after cleanup; the exact rendered scene and generator hashes are distinct
from the packaged scene and portable controller hashes in `manifest.json`.
Historical source code and render evidence are preserved under `history`.

`recipe.json` is the complete accepted authoring recipe: 3790 exact affine cell
placements, density coefficients, 86 union groups and voxel sizes. It samples
the 10 original VDBs actually used, retained byte-for-byte in `original-volumes`.
The original library contained 36 sculptures; unused originals are not required
to reproduce this scene. The current compiled VDBs reside in `cloud-volumes`.

Each continuous field is a bounded pointwise maximum of trilinearly sampled
original densities. All stored boundary faces are exactly zero. OpenVDB writes
a fresh file UUID during serialization; the recipe replays only that header
identifier, leaving all density/topology bytes untouched, to reproduce exact
cached file SHA-256 values. The CPU rebuild proof covers all 86 fields.

Run from the repository or package root, using Blender 5.2.1:

```text
blender --background --threads 4 tools/art/sky-source/sky-world.blend --python tools/art/build_sky.py -- --verify-scene
blender --background --threads 4 tools/art/sky-source/sky-world.blend --python tools/art/build_sky.py -- --rebuild-volumes
node tools/art/verify-sky.mjs --self-test
node tools/art/verify-sky.mjs
```

The rebuild writes scratch files under `.dream-loop/sky-volume-rebuild`; it does
not overwrite the authoritative grids or change the source blend. It verifies
all original input hashes, exact derived hashes, stored zero halos and bounded
finite density. `docs/evidence/sky-volume-rebuild.json` records the result.

Rendering requires a coordinated GPU window:

```text
blender --background --threads 4 tools/art/sky-source/sky-world.blend --python tools/art/build_sky.py -- --hero
blender --background --threads 4 tools/art/sky-source/sky-world.blend --python tools/art/build_sky.py -- --final
```

`--hero` renders 1672×941. `--final` renders an original 4096×2048 Radiance HDR,
then packages linear fp16 samples directly and updates provenance. It never
requests 8K. After an intentional saved scene edit, `--refresh` records the new
descriptor and sets `output_needs_render=true`; validation refuses stale output
until a new original final render completes.

`verify-sky.mjs` supports both historical 36-grid manifests and this composite
recipe. It checks every file hash, recipe coverage, rebuild proof, exact render
chain, KTX2 layout/linear DFD/values/alpha and sphere seam. It keeps the stable
`docs/evidence/sky-assets.json` path required by `verify-open-world.mjs`, while
archiving earlier source/output reports under `docs/evidence/sky-history`.
Optional `--root` and `--report` permit isolated checks without touching a
canonical evidence file.

The golden opening remains an unresolved visual target. File verification and
scene equivalence do not certify a match to the reference or runtime performance.
