# Regional cloud field r47 / reviewed recipe r2

This portable tree is ready for parent promotion. It changes two actual 3D
density/normal caches and their generators/controls. No shader, Rust source,
runtime lighting, bank location, floor, collision, screenshot or GPU profile is
included in the change. The original broad bank lobes are replaced by connected
240 m plume groups, two offset crowns per stem and two smaller anisotropic folds
per crown. This is a traversable volumetric field, with no baked lighting.

## Accepted approximation contract

The parent accepted the regional approximation explicitly after the native r47
Dawn capture and four matched CPU ray comparisons. **The historical regional
p99<0.035 test still fails.** Its sources, config, output reports and old caches
are preserved under `archive/`; no earlier proof was edited to imply success.

For the new regional recipe only, the accepted contract is pointwise density
mean<0.006 and p99<0.05, analytic-normal derivative p99<0.05 degrees, plus opaque
display RMSE<0.025 and edge-opacity p99<0.08 on all four fixed comparison views.
This makes the cost of sampling smaller 80–150 m folds at 20 m explicit and guards
it with independent image and derivative measurements. High banks retain the
historical .006/.035 bounds and exact former bytes.

| Pointwise metric | Hollow | Underforge |
| --- | ---: | ---: |
| Mean density error | 0.004527 | 0.004552 |
| Density error p99 | 0.043331 | 0.043167 |
| Surface potential 0.12–0.65 p99 | 0.059633 | 0.055508 |
| Crevices potential <=0.05 p99 | 0.029428 | 0.031288 |
| Analytic normal vs finite difference p99, degrees | 0.03426 | 0.03432 |

Exact quantized function values at 2,048 texel centers per bank, 36,000 random
points per bank over four hours of bounded advection, and cell-ownership seams
are audited. Across +/-0.01 m at ownership boundaries, maximum density delta is
about 0.00051. The normal recipe is coherent with the same scalar function;
interpolation, rather than a seam or inconsistent derivative, causes the errors.

| Matched-ray view | Opaque display RMSE | Opaque mean absolute | Edge opacity p99 |
| --- | ---: | ---: | ---: |
| Hollow oblique | 0.01912 | 0.01022 | 0.04301 |
| Hollow grazing | 0.02189 | 0.01040 | 0.07571 |
| Underforge oblique | 0.01901 | 0.01041 | 0.04323 |
| Underforge grazing | 0.02170 | 0.01012 | 0.07123 |

The comparisons integrate identical rays with analytic density/normal, cached
density/normal, and cached density with analytic normal. Empty background is
excluded from the opaque/edge measures. The displayed opaque p99 error is
0.0773–0.0938, with an isolated maximum 0.3872. Local lit joins therefore retain a
real approximation error. The images preserve overall peaks and silhouettes
closely; they do not claim identity. The live erosion field is fixed to the same
values in every pass, so this is a macro-field fidelity study, not a duplicate
of the complete engine sky, exposure or temporal pipeline. Float arrays and
the twelve computed images are retained in `review/`.

## Memory, integrity and reproduction

Two 256×96×256 RGBA8 regional fields use 24 MiB each. Four 96×64×96 high fields
use 2.25 MiB each: **57 MiB GPU total**, previously 27 MiB. The contract fixes every
center, extent, seed, resolution, padding and shape component. In particular,
`shape.w=1` and the runtime continuous floor remain unchanged above the caves.
The four high banks are byte-identical to the archived historical assets.

From the repository root, the existing no-argument gate remains:

```powershell
node tools/art/verify-cloud-cache.mjs
```

For a full CPU rebuild and fresh fidelity proof (Blender 5.2, at most 4 threads):

```powershell
$blenderExe = 'C:/Program Files/Blender Foundation/Blender 5.2/blender.exe'
& $blenderExe -b -t 4 --python-exit-code 1 --python tools/art/build_cloud_cache.py
& $blenderExe -b -t 4 --python-exit-code 1 --python tools/art/verify_cloud_cache.py
node tools/art/verify-cloud-cache.mjs
```

The Python gate recalculates the pointwise/derivative study and all twelve CPU
views, then applies the accepted contract. The Node gate checks actual cache
bytes, exact reviewed recipe/configuration, 57 MiB, immutable archives, source
hashes and the fresh proof bindings. Changing a source invalidates its proof.

`docs/evidence/cloud-r47-relocation-rebuild.json` proves a real rebuild into an
empty relocated tree: all six output SHA-256 values exactly match the reviewed
r2 caches. The generator was copied without edits and outputs were generated,
not copied. The ray proof takes about 63 seconds here, plus the cache bake.
`cloud-r47-negative-tests.json` records 13 mutations correctly rejected: memory,
floor, high/regional data, generator, archive, density/derivative, image/opacity,
camera and stale source proof. The test script requires an explicit isolated
scratch path and copies only cloud-specific dependencies.

No GPU performance claim follows from unchanged ray sample counts: larger caches
may affect bandwidth and locality. Native profiles remain parent-owned. The
runtime assets are solely `assets/atmosphere`; archived bytes and review float
arrays are authoring evidence and consume no GPU memory.
