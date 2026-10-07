# R44 portable rebuild: tangent diagnosis

Historical investigation, integrated on 3 October 2026. The installed validator and supplemental rounding evidence are one directory above this file. Run `node tools/art/foliage-r44-rebuild/test-verifier.mjs` for the portable corruption checks, then `node tools/art/verify-open-world.mjs` for the complete model audit. The diagnosis below describes the isolated work before its reviewed installation.

The rebuild does not change authored geometry. All 26 models were compared to the authentic historical candidate GLB, whose file SHA256 was first checked against the frozen baseline. Positions, indexed corner order, normals, UVs, triangle counts, node transforms, pivots, materials/factors/textures and all three collision/landmark JSON files are exact. All LOD attributes are exact. Four foliage primitives in three HD models differ only in the tangent attribute.

| GLB / material | Changed corner components | Unique vertices | Maximum angle |
|---|---:|---:|---:|
| Dawn / 08 | 9 | 3 | 0.005384776° |
| Dawn Garden / 08 | 4 | 1 | 0.005263362° |
| Dawn Garden / 09 | 3 | 1 | 0.004307943° |
| Dawn Ruin / 08 | 4 | 1 | 0.001405750° |

Total: 20 indexed corner components, representing six unique vertices. Each changed component moves between adjacent four-decimal bins. Maximum absolute difference is `0.00010001659393310547`, corresponding to 1678–3356 float32 ULP at these magnitudes. **It would be incorrect to describe the final GLB difference as one ULP.** Tangent W/handedness is exact everywhere.

## Cause and CPU reproduction

The installed Blender 5.2.1 exporter computes mesh tangents, then applies `numpy.round(self.tangents, ROUNDING_DIGIT)` in `io_scene_gltf2/blender/exp/primitive_extract.py:1529`. `ROUNDING_DIGIT = 4` is defined in `io/com/constants.py:160`.

`raw-tangents.py` opened the historical and rebuilt Blender sources read-only. Their raw positions, loop indices, UVs and corner normals are identical for every foliage mesh in the three affected islands. Recalculated raw tangent components differ by at most `1.7881393432617188e-7`. At threshold-crossing examples the raw difference is one or two float32 ULP, but four-decimal rounding magnifies it into one full 0.0001 quantization step. For example, `0.9537498950958252` versus `0.95374995470047` rounds to `0.9537` versus `0.9538` in NumPy float32 arithmetic.

`repeat-tangents.py` recalculated tangents eight times on fresh copies of the same saved canonical mesh, without changes to any input. The four affected foliage meshes each produced eight distinct raw tangent hashes. Rounded hash counts were five for Dawn 08, six for Garden 08, two for Garden 09 and two for Ruin 08. The two other tested foliage meshes were stable. Raw normals remained hash-identical across copies. This reproduces numerical nondeterminism in the CPU tangent calculation; the precise native scheduling/accumulation cause below Blender's API was not established. It is not a vertex-order change or a material-path effect.

No source Blender file was saved, no GLB was exported, and no rendering or GPU work occurred. The probes only wrote JSON and NumPy evidence under this diagnosis directory.

## Proposed verifier correction

Two local files are ready for parent review and later installation into `tools/art/`:

- `verify-foliage-materials.mjs`: proposed replacement verifier.
- `foliage-r44-tangent-rounding.json`: supplemental historical evidence, leaving `foliage-r44-baseline.json` byte-identical (SHA256 `697164c2f6ca62083b2be70e79948aecbdc155241f53b3b387a4c6d7770d1f92`).

This is a sparse exception, not a general epsilon. It permits only the historical or measured rebuilt value at exactly the 20 listed corner/component coordinates in the four named primitives. They must be adjacent four-decimal bins, differ by at most `0.00010002`, and produce a tangent angle at most `0.006°`. Every other component, including W, must still reconstruct the original historical SHA256 exactly. The proof binds each primitive to its original baseline tangent hash and checks the baseline file hash itself. The actual rebuilt tangent hashes and measured differences remain explicit in the validation report; it does not claim tangents are byte-identical.

The exception also requires foliage without a normal texture or anisotropic material extension. Those are the current affected materials, whose visible shading uses the unchanged normals. There is no reason to alter exporter geometry or regenerate assets to remove this measured numerical noise. Any future change elsewhere, including another rounding-boundary vertex observed in a later export, will fail strictly and require its own bounded evidence. The broader set of boundaries seen by repeated raw probes is intentionally not silently accepted.

## Validation

The proposed verifier passes against all 26 canonical rebuilt world models with original manifests/source hashes and native textures. `proposed-validator-report.json` records the four exceptions. `test-verifier.mjs` accepts all four authentic historical/rebuilt pairs and rejects 28 mutations: unlisted one-ULP changes, unmeasured values within an apparent epsilon, two-bin shifts, handedness flips, normal-mapped foliage, non-foliage materials, and anisotropic foliage.

Reproduce without installing anything:

```powershell
node .dream-loop/foliage-r44/rebuild-diagnosis/diagnose.mjs
node .dream-loop/foliage-r44/rebuild-diagnosis/prepare-verifier.mjs
node .dream-loop/foliage-r44/rebuild-diagnosis/verify-foliage-materials.mjs --root . --rounding-evidence .dream-loop/foliage-r44/rebuild-diagnosis/foliage-r44-tangent-rounding.json --report .dream-loop/foliage-r44/rebuild-diagnosis/proposed-validator-report.json
node .dream-loop/foliage-r44/rebuild-diagnosis/test-verifier.mjs
```

The Blender probe scripts must be launched hidden with `--background --factory-startup --python <script>`; they never invoke render/export. After an authorized installation of the two proposed files, the parent's regular `node tools/art/verify-open-world.mjs` can run all 34 checks. That full canonical command was not run here because it writes canonical evidence files; this task remains isolated.

The canonical generators, assets, manifests, baseline, Rust, WGSL and target outputs were not modified by this diagnosis. Sources still match the portable promotion hashes. No Git commands were used.
