# R62 — coherent native current, one isolated prototype

**CPU only. No native compilation, visual score, GPU timing or promotion claimed.** The package lives on D because the system drive reported zero free bytes. It contains only the shader, untouched12MB-class native texture sources, historical shaders and small CPU evidence; no copied world stage. Native capture must use the parent's current final R59+R60 assets, sky/weather and the exact comparable camera. Do not reuse early sourceC world metadata.

## Structural diagnosis and decision

R61 has a readable spiral eye and blue/violet again, but the foreground path remains faint and made of many equivalent threads. The old `liquid_sample` mixes two portions of the native image separated by half a tile over the entire longitudinal period. It then thresholds their averaged color. That overlays networks and attenuates peaks before selection. The same blending function remains byte-exact for waterfalls; R62 adds a separate current optical sampler.

At native linear energy>.09, the R61 crop.42+.29 has19.58 strong transverse segments on average, median19.5. Crop.16+.42 has7.125, median8. These are actual PNG/KTX counts, not projected2–8px widths or a guarantee of seven visible rails. The sample remains the real irregular authored network. Broad routes use the new crop; the narrow thermal keeps.42+.29 through the existing inverse UV→world Jacobian. Routes, UV buffers, positions, mesh topology, collisions, gameplay, pause/reduced-motion clock and native PNG/KTX bytes remain untouched.

The new structure is:

1. Classify body, crest radiance and coverage on each original texture sample before either blend.
2. Restrict the half-tile seam substitute to the boundary13% of longitudinal UV; the middle87% reads one native network. The boundary substitution still preserves continuous non-tileable sampling without editing pixels.
3. Blend two temporal phases offset by half a6-second period, with centered displacement±.0175U and at most±.0015V per phase. Existing longitudinal scroll-.10t and the project clock remain. A bounded interior lookup of the native blue coarse field modulates phase weakly; no external noise, authored flow-map claim, normal or height map is invented.
4. Give native strong crests enough coverage against cloud whites, independent of a deep blue body at1.8–3.3% local coverage. Coverage uses fixed smoothstep(.05,.18,ridge); radiance retains the historical adaptive threshold. This prevents distant low-pass averages from becoming an opaque sheet.
5. Preserve the physical width classifier, central optical envelope and terminal-core ramp. Existing gain14 stays; spectral soft rolloff bounds individual crest mass at6 before the separate compact-core contribution. That can weaken the previously acquired eye, so the native15–25px core is a required capture gate, not assumed preserved visually.

The first CPU draft reused the adaptive radiance threshold for coverage and reached27.7% mean alpha at mip8. It was rejected before any GPU rendering and is preserved under `history/pre-coverage-conservation/`. The current prototype reduces this to1.80%; mip6 mean is3.00%. This is an explicit new coverage contract, not a relaxation or relabeling of R58's alpha-exact historical gate. Both R53/R58/R61 shaders remain pinned and unchanged.

## Measurement and limitations

The revised CPU material uses exact KTX levels0–8, linear sRGB decoding and bilinear/trilinear sampling. Relative to R61, mean associated luminance is0.775–1.180× across those fixed mips. This is redistribution and background coverage, not a blanket gain increase. Native long-crest widths, temporal coherence, cyan/violet after Bevy tone mapping, bloom and depth sorting require actual engine screenshots. There is no CPU swatch masquerading as gameplay and no bitmap output or post-retouching.

Current fetch budget is **five**: two gradient samples for each temporal phase plus one coarse native carrier sample. Waterfalls keep their original two. There are no derivative operations or loops in new sampling helpers; gradients come from the caller. The existing material-selector branch is uniform. Jacobian tests cover512 random rotations/scales/both windings; seam/temporal tests bound the observed CPU continuity delta below1e-4. Static checks are not a WGSL compiler. The extra fetches and fragment math must be measured on native and browser; no performance benefit is presumed.

All current alpha/sampling changes are declared. Waterfall/pool/haze, source bindings, vertex alpha and clock restore exactly to the pinned historical shader after removing the current delta. No canonical Rust or other shaders are changed. Derivative footprint approximation ignores the tiny coarse-field phase-warp derivative; grazing/LOD motion must check filtering stability. Geometry normals are recovered from actual position derivatives because the mesh's authored normals are flatY.

## Reconstruct and check, with no GPU

```text
python tools/art/flow-r62-source/build_shader.py
python tools/art/flow-r62-source/probe.py
node tools/art/flow-r62-source/verify.mjs
python tools/art/flow-r62-source/test_portability.py
```

The last test relocates only this small package under D:/Aether-art-studies, verifies its absolute temporary path before cleanup, rebuilds identical shader bytes and reruns the probe and ten negative contract cases. It never writes into the canonical project. No Git actions or native captures are launched by this author.

## Primary research

[Alex Vlachos, Valve SIGGRAPH2010: Water Flow](https://cdn.fastly.steamstatic.com/apps/valve/2010/siggraph2010_vlachos_waterflow.pdf), slides22–23,31–37,45–46, describes short distortion, two half-phase layers, repetition/pulse control and a centered distortion interval for flowing color. R62 adapts those ideas to the existing color texture and actual route UV, without claiming a real authored vector-flow map or the published hardware performance.

[GPU Gems, chapter1: Effective Water Simulation from Physical Models](https://developer.nvidia.com/gpugems/gpugems/part-i-natural-effects/chapter-1-effective-water-simulation-physical-models), section1.2, separates geometric undulation and finer texture waves. Here the existing macro geometry stays, while optical crest sampling changes. The chapter's geometric wave/normal generation is not used to manufacture new rails or a false normal map.

## Next root capture gates

- Compile the actual candidate WGSL with the final native gallery; validate shader SHA and zero renderer errors.
- Same camera/time/lighting versus R61: a coherent foreground stream, fewer2–8px highlights, blue depth, violet secondary, dark gaps and no large gray/opaque patches.
- Preserve a15–25px spiral eye and continuous curved native arcs; compare close, grazing and moving time, then browser/native frame costs.
- Only after those measurements consider an explicit independent finishing overlay. Historical gates and thresholds remain untouched, and promotion is not authorized by this package.
