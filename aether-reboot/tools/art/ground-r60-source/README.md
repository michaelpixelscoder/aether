# R60 — natural garden ground, isolated PNG prototype

**Final portable R59 rebase, not promoted by this author. Exact portable wide/Watch-close native captures and filtering PASS; visual score, gameplay and GPU timing remain open.** This package is distinct from the original study; it uses the final R59 caller metadata/hashes77c382…/be0f58… rather than the earlier study's8372c8…/ecead228…. The earlier study/prototypes remain untouched. Prior native close capture exists with the earlier caller metadata, and its source/UV optical state is the same; it cannot be mislabeled as an exact portable-hash capture. This is an albedo-only prototype for Dawn/Watch HD and LOD material12.

## Source and scope

The original ImageGen PNG is `source/ground-r60-native.png`,1254² RGB8, SHA`087deb4c68767e876653edc1b1fd0890b0ce5cb051beb5a29d03de718806b365`. The exact generation prompt and root metadata are `source/prompt.txt` and `source/generation.json`. Built-in `image_gen.imagegen`, transparent_background=false, no reference image or edit. A4×4m orthographic natural short-grass/moss garden patch was requested with neutral flat illumination, fine botanical detail, no baked shadow, checkerboard, objects or text. The native source is untouched: no pixel retouching, color remapping, resize, invented height or normal map.

Runtime URI is content-addressed `textures/087deb4c68767e876653edc1b1fd0890b0ce5cb051beb5a29d03de718806b365.png`. Every source-file byte matches the original. The versioned author PNG retains its readable filename. Standard glTF PNG encodes one source level, but this does **not** imply one runtime mip: actual `crates/aether_view/src/mipmaps.rs::prepare` receives opaque RGBA8(sRGB) image events too and generates an odd-size linear-sRGB mip chain, with anisotropy8. Both the earlier fixture and the final portable native capture measured11 levels and byte-exact level0 for1254². This package does not invent an incorrect KHR_texture_basisu claim for uncompressed KTX2; the unmodified R53 encoder archive is only a historical reference.

Only12's base-color texture binding, explicit neutral linear factor[1,1,1,1] and KHR_texture_transform scale change. Roughness~0.88, metallic0, double-sided behavior, other material properties, all other11 HD/9 LOD materials, existing image/texture/sampler entries, nodes, meshes, accessors, UV, indices and the entire BIN chunk remain exact. A new image/texture entry is appended; old indices stay stable. Existing architecture/terraces/cliff extras and collision/landmark historical manifest metadata remain exact. The manifest updates only the four actual high/LOD bytes/SHA pairs and adds `ground_r60` provenance under Dawn/Watch; all other fields and island entries remain exact. This is a selective merge onto the authenticated R59 manifest, not a transplant of an older whole manifest. No GLB extras are changed.

## Actual color and scale measurements

Source palette declares turf linear(.095,.19,.032). This is distinct from the actual runtime state: all four inspected GLBs omit baseColorFactor (default1), and **every material12 primitive has no COLOR_0 attribute**. Therefore neither the source palette value nor a hypothetical vertex tint is blindly multiplied into this new albedo. If future callers contain vertex color, the complete BIN preservation keeps it unchanged; its additional tint would need its own measurement before promotion.

Actual old grass PNG is256² and visibly tiled into colored squares. Its mean linear luminance is0.120799. The new natural1254² source mean is0.096198 (~20.4% lower), with RGB means(.09139,.10454,.02770). Neutral factor1 preserves those real albedo values; glTF factors above1 are invalid. No exposure increase or pixel compensation hides the darker natural source. Final dawn lighting must establish whether it remains readable.

Measured floor UV singular values are0.10999984–0.11000012 per local metre, matching the generator's0.11 metric UV. Existing repeat size is9.0909m locally,19.0909m at the primary Dawn/Watch instance scale2.1. Material transform scale`2.1/(4×0.11)=4.7727272727` produces the requested4m repeat on those primary instances while preserving all UV bytes and rotations. Reused Dawn instances at scale1.5 have a2.8571m repeat; a single shared material cannot keep an absolute4m repeat across variable instance scales without extra runtime state. That limitation is explicit, not concealed by changing geometry.

## Portable application

Pass a real input-world directory containing all four GLBs and its manifest; the output must be a different directory:

```text
python tools/art/ground-r60-source/patch_ground.py --input-world tools/art/ground-r60-source/history/r59-portable-world --output-world assets/world --report tools/art/ground-r60-source/proofs/r59-portable-patch.json
node tools/art/ground-r60-source/verify.mjs
node tools/art/ground-r60-source/test-api.mjs
python tools/art/ground-r60-source/probe_ground.py
python tools/art/ground-r60-source/test_portability.py
```

The generator accepts any explicit distinct input/output world directory, never a fixed old model transplant. It locates the caller's unique material12 by name and appends the new binding to that exact GLB. It refuses already-patched or unexpectedly transformed textures. For arbitrary callers, invoke the independent API with their separately authenticated expectedInputSha and nativePNG; the production CLI deliberately verifies this package's final pinned R59 caller rather than guessing a new historical baseline. The older study proved independent R56/pre-finalR59 reconstruction; the final relocation proves all four outputs over this final portable caller. No Blender export is used.

This package's `assets/world` is final portable R59+R60. Its R59 Dawn input is77c38267abdbce3208f891b1e472a6bc522eccdee7fcf5e62402a7721449acc4; Watch input isbe0f58ae2c3a8d91b18b0b994b69c97f83ecc042cb0578b07f1faf39b92092f6. All newer R59 metadata and cliff batches survive. They are frozen under `history/r59-portable-world`, and their high SHA authority is the unchanged R59 `assembly-evidence.json`, copied to `source/r59-assembly-evidence.json` and pinned atSHAe6035440828c3c45753ff6c1c46feee4d6f263a2752a2fb4897f58ff27e575cd. The old R56 and pre-final study baselines remain historical archive data.

Independent API `validate-ground-delta.mjs` exports `validateGroundDelta(beforeRaw,afterRaw,{expectedInputSha,nativePNG})`. The caller's independently authenticated64-hex SHA is mandatory. Every BIN byte and every unrelated JSON field is checked before any projection is returned. `projectedDoc` restores just the proven material/image/texture/extensionsUsed patch, and `bin` is the actual unchanged candidate binary. R54/R56 compatibility overlays are owned by the cliff author and may consume this API after their own R59 authentication; this package does not edit those overlays. `validate-world.mjs` provides a whole-world R60 audit using the pinned final R59 assembly evidence and fixed original LOD hashes. It verifies the real four-file runtime delta and preserved Dawn/Watch historical metadata.

The final R59 `verify-open-world.mjs` overlay already includes the optional R60 gate and is retained byte-exact (SHA351f13e9b65d320e2b911c06f82779fcdca90d3fa6636ff30b31415dd3cabba6). The isolated foliage overlay invokes the independent whole-world delta audit before the unchanged historical R44 06/08/09 family checks. No family threshold or tangent evidence changes. All old gates are archived. Island/variant/mineral/masonry gates already accept content-addressed PNG and target their own family; they need no R60 exception. Final global execution over the fully composed native world remains an integration gate; historical R54/R56/R59 overlays are owned by the cliff author.

## Actual native filtering evidence, separate from final portable capture

The parent measured the original PNG in the earlier R60 native fixture: `source/native-runtime-filtering-study.json` is an exact copy of that report. Actual1254² RGBA8sRGB, repeat/linear min/mag/mip, anisotropy8,11 mip levels and8,384,072 texel bytes were observed. Native level0 RGBA SHA`c0f56f9e7a9b15f8f39fe1024cc161e861db8108ba548c716d256f6b991b4c59` equals the decoded source RGB→RGBA SHA; no pixel change occurred. These are measurements of the earlier fixture, whose GLB metadata predates the final portable R59 caller. The report is not reclassified as a final portable capture, GPU timing, VRAM residency, or gameplay proof. Older proofs remain archived unchanged.

The untouched capture PNG, actual surface report, level0 RGBA dump and renderer evidence are archived under `source/native-filtering-study/`. The gate checks their SHA, zero renderer errors, exit0 and the actual earlier Dawn GLB SHA83633442596ede95400e6a746d8dd9d831824a6b2b183960dfd7de9e010b3f77. This preserves a portable proof without relabeling it as final. The root capture artifacts remain untouched.

## Final portable native evidence

`source/native-final-portable/` separately preserves the root's exact final wide and Watch close captures, their renderer evidence, final surface report/RGBA dump and `r60-ground-final-runtime-filtering.json`. The production gate verifies the actual four final GLB SHA in both capture inventories, PNG hashes, zero renderer errors, exit0, native1254² RGBA8sRGB,11 mips,8,384,072 texel bytes, repeat/linear/anis8 and byte-exact level0. This is native art-fixture evidence; GPU frame timing and gameplay remain unproved.

Both images were inspected. Dawn/Watch ground is a natural, subdued olive surface with finer non-square variation; the old mottled checker pattern is removed from the intended four models. Close Watch still reads relatively flat at the capture distance because this is only albedo, without blade geometry or a normal map. The wide view still shows other island variants with their existing ground material, since R60 only patches the agreed Dawn/Watch quartet. No final-fidelity score is invented from these captures.

## Reviewable promotion plan

The sibling `promotion-plan.json` pins every file write and its expected preimage after final R59. It preserves the R59 global verifier byte-exact. `merge-manifest.mjs` requires the independently authenticated R59 manifest SHA, clones the real caller, sets exactly eight high/LOD hash/bytes fields and two `ground_r60` entries, then rejects any unrelated proposal change. Six negative manifest cases are documented alongside the14 negative material/API cases. Author/source hashes, native PNG URI, neutral factor and4m-primary transform appear explicitly in each new provenance entry. The plan generator writes only into this isolated study; it performs no promotion.

## Remaining native gates

- Same-camera original-source overlay, with final8K/R59 lighting fixed: confirm grass reads natural green, does not go black/yellow, and does not regress the accepted architectural composition.
- Check HD→LOD color continuity, grazing-angle movement and repeated-edge visibility. Final portable11-level mip chain and native level0 hash are measured and verified; this does not establish temporal shimmer or frame performance.
- Measure texture memory/performance. Shared PNG level0 decodes to6,290,064 RGBA8 bytes; an11-level odd-size chain would use8,384,072 bytes. Actual image residency and frame costs remain runtime measurements, not accepted budgets inferred from those calculations.
- No normal map is added. A later genuine ImageGen normal asset may help; this package does not derive misleading normal/height data from color.
- Promotion requires explicit compatibility extensions in historical art gates, preserving their original baselines/thresholds and this separate albedo delta proof. Existing R54/R56/R59 gates have not been silently rewritten.

## Primary technical sources

glTF defines base-color texture as sRGB and its factor as a linear multiplier with default1 and allowed range0–1. This is why the source is preserved and no brightness factor above1 is used. [Khronos glTF2.0 specification](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#material-metallic-roughness).

Texture-transform scaling is a texture-info transform, which changes sampling without modifying mesh UV attributes. [Khronos KHR_texture_transform](https://raw.githubusercontent.com/KhronosGroup/glTF/main/extensions/2.0/Khronos/KHR_texture_transform/README.md).

Bevy0.19.1's actual pinned glTF loader reads base-color texture transforms into its material UV transform and loads base-color images with is_srgb=true. Local registry source was checked before implementation. [Bevy0.19.1 loader source](https://raw.githubusercontent.com/bevyengine/bevy/v0.19.1/crates/bevy_gltf/src/loader/mod.rs). No unverified GPU or community performance claim follows from that source check.
