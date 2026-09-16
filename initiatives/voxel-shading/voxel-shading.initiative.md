---
title: "Voxel Shading"
status: active
owner: michael
created: 2026-09-13
updated: 2026-09-13
target_date: null
budget: null
related_thoughts:
  - thoughts/voxel-shading/voxel-shading.thought.md
related_stories: []
related_actors: []
tags:
  - aether-isles
  - voxels
  - rendering
  - shading
  - textures
  - materials
---

# Voxel Shading

## 1. Problem

- **Affected actors:** Players who build with or explore voxel structures, and developers and artists who author blocks and modeled assets.
- **Current situation:** The voxel prototype greedy-merges exposed faces and applies one small tiled image through a Bevy `StandardMaterial` for each `BlockKind`. Repeated blocks read as a grid of generic cubes; merged faces stretch the same UV range; there is no stable way to express connected surfaces, macro variation, face orientation, borders, corners, or material-specific edge details.
- **Impact:** Ships and islands cannot reach the richer, continuous, natural visual language required by *Aether Isles*. Adding more block types now would also multiply ad hoc material and asset conventions.
- **Evidence:** The voxel-shading thought calls for less tiling, larger continuous surfaces, and a natural appearance, with connected armor in *Space Engineers* as a reference. The current shipwright prototype has only five flat texture-backed block kinds and no surface-neighborhood data in its rendered meshes.

The intended visual promise is:

> A voxel structure remains clearly buildable from blocks, while each material reads as one crafted surface rather than a wallpapered stack of cubes.

The first proof material is named **wooden plank**. **Natural wood** is reserved for a later, distinct material family rather than used as an alias.

## 2. Completion criteria

| Measure | Baseline | Target | Evidence source | Measurement window |
| --- | --- | --- | --- | --- |
| First material proof | Current wood is a single repeating `wooden_plank.png` | Wooden plank renders across 1×1, strips, rectangles, corners, and irregular voxel assemblies with continuous large-scale detail | Reference-scene captures and acceptance checklist | End of Phase 1 |
| Border support | No border classification or rendering | Exposed surface boundaries can select edge/corner details independently of the face interior; wooden plank can place nails or fasteners only in eligible border locations | Deterministic topology tests and captures | End of Phase 1 |
| Surface continuity | Greedy faces each receive the same 0–1 UV range | Adjacent coplanar blocks share stable surface coordinates without visible per-block texture restarts | Mesh attribute tests and reference-scene captures | End of Phase 1 |
| Variation stability | No macro variation | Macro detail is deterministic in voxel-body local coordinates and does not swim when an island or ship moves or rotates | Transform comparison test | End of Phase 1 |
| Extensible material contract | Material behavior is embedded in shipwright code | A data-driven material definition selects surface, border, and shading behavior without changing voxel storage | Catalog fixture and dependency review | Before Phase 2 material work |
| Asset organization | Textures live in one prototype folder; modeled blocks have no convention | Block families and modeled assets follow the documented source/runtime structure and stable IDs | Repository structure review | Before adding the second material family |
| Visual direction | No recorded comparison | At least one shading research cycle and one texture research cycle are documented with alternatives, observations, and a correction | Research records and decision log | Before generalizing beyond wooden plank |
| Automated visual validation | Rendering changes have no repeatable visual test artifact | One command renders all predefined voxel constructions to labeled PNGs and produces a contact sheet plus an HTML gallery | Generated render gallery | Every rendering iteration |
| Performance guardrail | Not measured for the new pipeline | Reference bodies remain interactive and record mesh generation time, draw calls, vertex count, and texture memory against the current renderer | Repeatable benchmark scene | Every implementation cycle |

All required completion criteria:

- [ ] Wooden plank proves macro texture continuity, orientation, and border-aware details on a representative voxel body.
- [ ] Nails are a material-configured border detail, not baked into every tile or hard-coded as universal voxel geometry.
- [ ] Surface classification handles face interior, straight border, outer corner, inner corner, and isolated/end-cap cases, or explicitly records why a smaller classification is sufficient.
- [ ] Shading and texture research each pass through prototype → capture → compare → correct at least once.
- [ ] An automated render-gallery test makes every predefined construction easy to inspect without manually rebuilding scenes or positioning a camera.
- [ ] Material definitions, block definitions, generated meshes, source art, and modeled assets have separate ownership and dependency boundaries.
- [ ] The system retains voxel-body-local coordinates so moving ships do not trigger remeshing or texture swimming.

## 3. Constraints and stop conditions

### Constraints

- **Time:** No date is assigned. Estimate later phases only after the wooden-plank slice is measured.
- **Budget:** No monetary budget is assigned. Use original or properly licensed references and assets.
- **Capacity:** Keep the first implementation to one opaque cube block material and a compact reference scene.
- **Technical or operational constraints:** Rust and Bevy; cube voxels remain the default; separate movable voxel bodies; greedy meshing may change but must retain internal-face culling; native and WASM targets must remain possible.
- **Must preserve:** Stable voxel simulation data independent of presentation; deterministic output; readable block scale; support for authored non-cube/model-backed blocks later.

### Stop or reconsider if

- [ ] Wooden plank requires a unique shader or mesher path that cannot be expressed as a reusable material feature after two correction cycles.
- [ ] Border classification causes unacceptable geometry growth or remesh cost on the benchmark body after two optimization passes.
- [ ] Macro detail makes block placement unreadable in two successive visual comparisons.
- [ ] Texture memory or draw-call cost grows per connected surface or per voxel rather than remaining bounded by material/chunk strategy.
- [ ] The project begins producing multiple final materials before the material contract and asset structure survive the wooden-plank slice.

When a condition is met, pause material expansion, retain the smallest successful visual feature, and redesign the surface contract or rendering technique. Do not hide a weak system by creating more texture content.

## 4. Possible solutions

### Option A — Border-aware material atlas on enriched greedy meshes

- **Description:** Extend generated vertices with voxel-body-local surface coordinates and topology metadata. A reusable Bevy material samples tileable micro detail, low-frequency macro variation, and atlas/overlay regions chosen for interior, edge, corner, and optional detail masks.
- **Expected effect:** Continuous surfaces and material-specific borders while keeping cube storage and greedy meshing.
- **Cost and effort:** Moderate changes to mesh output, a custom material/shader, topology tests, and a small authored texture set.
- **Risks:** Greedy quad boundaries may disagree with visual borders; per-vertex metadata may become crowded; filtering can bleed across an atlas.
- **How it would be tested:** Implement wooden plank on the canonical topology matrix, measure geometry and rendering cost, and compare shader versus geometry-driven border details.

### Option B — Geometry-decal borders and authored detail meshes

- **Description:** Keep a simpler base surface and generate thin border strips, nail decals, or instanced detail meshes along classified boundaries.
- **Expected effect:** Crisp borders and flexible physical-looking details with simpler texture sampling.
- **Cost and effort:** More entities, geometry, depth-bias handling, and lifecycle complexity.
- **Risks:** Z-fighting, draw-call growth, and details becoming detached during remeshing or destruction.
- **How it would be tested:** Use as the explicit alternative in the wooden-plank border experiment and compare cost and image quality with Option A.

### Option C — Conventional per-block tiling

- **Description:** Improve the current images but keep one texture restart per block or merged quad.
- **Likely consequence:** Cheap and readable, but macro repetition and disconnected surfaces remain.
- **When this is the correct choice:** As a fallback for prototypes, distant LODs, or materials that intentionally expose every block.

## 5. Selected solution

- **Decision:** Begin with Option A as a thin vertical slice, while retaining Option B as the measured alternative for nails and other sparse border details.
- **Why this option:** It addresses continuity and borders at the shared surface representation instead of specializing the wooden-plank block. It also fits independent moving voxel bodies because coordinates can remain local.
- **Assumptions being made:** Surface topology can be encoded compactly; macro and micro detail can coexist without hiding voxel scale; border decoration can be data-driven; the current mesher can be evolved without replacing voxel storage.
- **Known risks:** Texture atlas bleeding, ambiguous corners, directional plank orientation, seams across chunk boundaries, transparent/flexible materials needing different pipelines, and over-generalizing from wood.
- **In scope:** Opaque cube surfaces, local surface coordinates, face orientation, connected-surface topology, macro/micro texture layers, border masks, wooden-plank seams and optional nails, debug views, reference fixtures, research cycles, performance measurements, and project structure.
- **Out of scope:** Natural wood, stone, terrain blending, glass/transparency, liquids, animated damage/wetness/fire, smooth voxels, final LOD strategy, and full authored complex-block rendering. These become later research or material slices after the shared contract is validated.

## 6. Architecture and project structure

### Runtime responsibilities

```text
voxel storage (block IDs + state)
        │
        ▼
block catalog ──> shape/model reference + material assignments
        │
        ▼
surface extraction ──> face orientation + connected-neighbor topology
        │
        ▼
mesh attributes ──> body-local position + surface coordinates + border class
        │
        ▼
voxel material catalog ──> texture layers + shading/border rules
        │
        ▼
Bevy presentation material/shader
```

- A **block type** defines simulation identity, shape, face-to-material assignments, rotation/state, and optional modeled-asset reference.
- A **voxel material** defines how a surface looks: texture layers, physical parameters, scale, direction rules, borders, and detail placement.
- A **modeled asset** defines authored geometry and sockets/anchors. It references material IDs; it does not own duplicate material definitions.
- The **mesher** emits generic surface facts. It must not contain wooden-plank-specific branches.
- The **shader/material implementation** consumes generic facts and material parameters. Nails are configured by the wooden-plank material.

### Proposed repository layout

```text
crates/
  features/
    aether_voxels/                 # storage, topology, surface extraction, mesh data
    aether_blocks/                 # stable block IDs, definitions, shapes, face materials
  presentation/
    aether_voxel_render/           # Bevy mesh adapter, material plugin, debug views
assets/
  voxel_materials/
    wooden_plank/
      material.ron                 # stable ID and rendering parameters
      source/                      # editable source art; excluded from runtime loading
      textures/                    # runtime albedo/normal/roughness/masks/atlas
      references.md                # licensed references and visual decisions
  blocks/
    structural/
      wooden_plank_block/
        block.ron                  # shape, material-per-face, tags, state schema
        preview.png
    natural/
      natural_wood/                # later, distinct family
  modeled_assets/
    ship_components/
      example_component/
        asset.ron                  # stable ID, model, materials, anchors, collision
        models/
        textures/                  # only asset-unique maps; shared maps stay in voxel_materials
        source/
tests/
  fixtures/
    voxel_surfaces/                # topology matrices and benchmark bodies
  voxel_render_gallery/            # gallery scene manifest, camera and lighting presets
design/
  voxel-shading/
    research/                      # dated shading and texture experiment records
    captures/                      # comparable output from each cycle
```

The exact serialization format may change, but stable namespaced IDs should not be file paths, for example `aether:wooden_plank`, `aether:structural/wooden_plank_block`, and `aether:ship_components/propeller_small`.

### Initial material contract to validate

The wooden-plank definition should be able to declare:

- physical PBR defaults;
- world/body-local texel scale and plank direction rules per face;
- micro texture layers and macro variation scale/seed;
- border width and eligible topology classes;
- border texture/normal/roughness overrides;
- optional details such as nails, their eligible edges/corners, spacing, inset, and deterministic seed;
- debug colors and a fallback material.

Do not freeze this schema before the first two wooden-plank cycles. Record changes and why they were needed.

## 7. Phased implementation plan

Every research or material task uses the same loop:

```text
question → alternatives → smallest prototype → fixed-scene captures/metrics
         → review → one correction → decision recorded
```

No research task is complete at “collect references,” and no material task is complete at its first render.

### Phase 0 — Establish evidence and contracts

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Canonical surface fixtures | Saved bodies covering single voxel, 1×N strip, rectangle, concave/convex corners, holes, steps, chunk seam, and rotated/moved body | Current voxel storage | completed | Eight deterministic fixtures render from fixed cameras; a true multi-chunk meshing fixture remains part of later chunk work |
| Automated voxel render gallery | Test application loads every fixture, waits for assets/pipelines, renders fixed views to PNG, then builds a labeled contact sheet and HTML index | Canonical surface fixtures | completed | `./scripts/render-voxel-gallery.sh` creates the complete gallery at `target/voxel-render-gallery/index.html` and exits non-zero on capture/finalization failure |
| Baseline diagnostics | Current frame, mesh time, vertices, indices, draw calls, and texture memory recorded | Fixtures | pending | Machine-readable or dated baseline is stored |
| Shading research cycle 1 | Compare at minimum directional/ambient lighting, voxel AO or curvature cues, and macro color/roughness variation against the target art direction | Fixtures and concept designs | pending | Research note contains alternatives, captures, review, correction, and decision |
| Texture research cycle 1 | Compare projected/body-local mapping, face-oriented UVs, atlas/array strategies, macro/micro layering, filtering, and border masks | Fixtures | pending | Research note contains alternatives, captures, review, correction, and decision |
| Generative PBR workflow research | Compare open-source AI generation, seamless tiling, coherent map generation, PBR authoring, and Bevy export workflows | Cycle 1 texture evidence | completed | `design/voxel-shading/research/generative-pbr-texture-workflow.md` records tools, licenses, selected architecture, risks, and experiments |
| Draft block/material/model contracts | Separate stable definitions with dependency direction and loader strategy | Research findings | pending | Wooden-plank fixture can be described without shipwright-specific code |

### Phase 1 — Basic wooden-plank vertical slice

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Rename prototype material | UI, IDs, fixtures, and labels use `wooden_plank`; migration/alias decision for old `Wood` is recorded | Draft catalog | completed | Shipwright uses `BlockKind::WoodenPlank` and displays “Wooden plank”; natural wood remains unassigned |
| Surface-coordinate mesh data | Stable body-local coordinates and face orientation survive greedy merging and body transforms | Texture research | completed | Mesher emits separate absolute body-local surface coordinates; coordinate and greedy-span tests pass |
| Border topology | Faces classify exposed straight edges, corners, inner corners, and end/isolated cases across neighboring voxels and chunk boundaries | Surface fixtures | pending | Table-driven topology tests pass |
| Wooden-plank texture v1 | Large-scale plank layout plus micro grain reads continuously across connected faces | Surface coordinates | completed | Eight fixed captures and a corrected 4×4-voxel texture with visible board bevels and nail details are recorded in the Cycle 1 research note |
| Wooden-plank borders v1 | Material can render seams/edge wear and enable nails only at configured border locations | Border topology | pending | Nail-free interiors and deterministic eligible borders are verified |
| Shading v1 | Selected lighting/AO/macro response is implemented with debug modes | Shading research | pending | Debug and beauty captures exist for every fixture |
| Wooden-plank iteration 2 | Review v1 for repetition, orientation, borders, corners, and block readability; implement one correction pass | Integrated v1 | completed | Matched captures record the texture corrections; topology-aware border behavior remains explicitly deferred to Cycle 2 |
| Border technique decision | Atlas/overlay and geometry/decal approaches are compared for nails | Borders v1, metrics | pending | Chosen technique and rejected alternative are documented |
| Performance comparison | New pipeline measured against baseline on small and representative bodies | Integrated slice | pending | Regression is quantified and accepted or triggers correction |
| Generative wooden-plank PBR experiment | Recreate and improve wooden plank through a reproducible AI workflow, compare seamless techniques and map-generation backends, then perform one correction cycle | Generative PBR workflow research, render gallery | in progress | Pinned ComfyUI/TextureAlchemy workflow, source prompt/resources, base color, OpenGL normal, height, AO, roughness/metallic inputs, packed ORM, exact-edge report, and tiled preview are stored; a genuine 16-bit height path remains to test |
| PBR runtime integration | Wooden plank uses base color, normal, and packed ORM with matching repeat samplers and UV transforms in shipwright and the gallery | Generative wooden-plank PBR experiment | in progress | Gallery now loads linear normal/ORM maps with repeat samplers, generates mesh tangents, and passes the eight-fixture render run; Shipwright integration and multi-light comparison remain |

Phase 1 releases only when the wooden-plank slice is useful in shipwright and the generic contracts contain no wooden-plank-specific code.

### Phase 2 — Generalize through contrasting materials

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Texture research cycle 2 | Apply the contract to a contrasting surface such as cut stone or metal plate, including its own border language | Phase 1 | pending | First attempt, review, correction, and schema changes are recorded |
| Shading research cycle 2 | Test material response differences, distance behavior, and moving-body lighting | Phase 1 | pending | Captures and metrics result in a recorded decision |
| Natural wood exploration | Define natural wood as bark/end-grain/organic flow rather than plank construction | Generalized contract | pending | It has distinct reference, behavior, and naming; one corrected prototype exists |
| Catalog validation | Load multiple material and block definitions with fallbacks and actionable errors | Second material | pending | Automated valid/invalid catalog fixtures pass |

### Phase 3 — Block families and modeled assets

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Block-family migration | Shipwright block choices move from its private enum to stable catalog definitions | Validated catalog | pending | Editor behavior round-trips stable IDs |
| Modeled-asset adapter | Authored shapes reference shared materials and declare orientation, anchors, and collision metadata | Project structure | pending | One modeled block renders beside cube blocks without duplicating materials |
| Mixed-boundary research cycle | Explore how cube surfaces meet modeled blocks and whether borders terminate, continue, or use authored seams | Modeled-asset adapter | pending | At least two alternatives, correction, and decision recorded |
| Destruction/edit validation | Borders and macro textures remain deterministic after add/remove/split operations | Mixed renderer | pending | Fixed edit sequence produces stable expected topology and captures |

### Phase 4 — Production breadth

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Material backlog prioritization | Terrain, glass, metal, fabric, wetness/damage, and magical materials are ordered by technical uncertainty and game value | Earlier evidence | pending | Each candidate has a question and smallest experiment |
| LOD and distance research cycle | Continuous surfaces retain character without shimmer or excessive cost across distance | Representative scene | pending | Corrected prototype and thresholds documented |
| Authoring workflow | Validation, previews, capture automation, and asset import are usable without modifying renderer code | Stable contracts | pending | A new material is added from definition/assets and passes checks |
| Production integration | Ships and islands share the same rendering contracts with bounded per-body cost | LOD and authoring workflow | pending | Representative ship and island pass visual and performance budgets |

## 8. Test instructions

### Prerequisites

- Stable Rust toolchain and repository assets
- Native GPU-capable build; `wasm32-unknown-unknown` for browser validation
- Canonical voxel-surface fixture scene introduced in Phase 0

### Automated verification

The visual test is a deterministic render-gallery generator rather than an interactive demo. Its checked-in manifest defines the voxel cells, block/material IDs, body transform, camera, lighting preset, resolution, and expected output name for each scene. At minimum it renders:

- a single voxel from an angle that shows three faces;
- a long row and a broad coplanar wall for continuity and repetition;
- an L-shaped outside corner and a concave inside corner;
- steps, a hole/window, isolated end caps, and a mixed-height irregular construction;
- a construction spanning a chunk boundary;
- the same construction translated and rotated as a movable voxel body;
- a small ship-hull-like construction that combines the cases above.

Each scene produces a labeled PNG. The test also creates `contact-sheet.png` and `index.html`, grouping beauty and debug renders by fixture. Debug variants should include surface coordinates, face orientation, border class, and nail/detail eligibility. Generated artifacts live under `target/voxel-render-gallery/` so normal test runs do not dirty the repository. A later phase may add approved golden-image comparison, but the first version exists to produce consistent, easily viewed evidence rather than fail on harmless GPU-level pixel differences.

Run from the repository root:

```bash
cargo fmt --all -- --check
cargo test -p aether_voxels
cargo check --workspace
cargo run -p aether_voxel_render_tests -- --output target/voxel-render-gallery
./scripts/build-web.sh
```

`aether_voxel_render_tests` is the proposed test-harness crate name; record the final command here if the workspace adopts a different name. The renderer must not capture until all fixture assets and render pipelines are ready. It must use fixed resolution, color space, camera transforms, lighting, seeds, and warm-up frame count, and write a machine-readable manifest beside the images.

As the renderer is introduced, also add deterministic data tests for surface coordinates, every border topology class, neighbor data across chunk boundaries, stable seeds, catalog validation, and unchanged results under voxel-body transforms.

Expected result:

- Exit code: `0`
- Expected output or generated artifact: Workspace checks pass; topology fixtures match expected classifications; every declared scene has a non-empty PNG; and `target/voxel-render-gallery/index.html`, `contact-sheet.png`, and the machine-readable render manifest are generated.

### Human acceptance checks

- [ ] Wooden planks form a coherent larger surface without looking like one image stretched over the whole object.
- [ ] Individual voxel scale remains legible during building and inspection.
- [ ] Grain/plank direction looks intentional on top, side, and end faces.
- [ ] Borders describe actual surface boundaries and do not appear on hidden/internal faces.
- [ ] Nails are sparse, deterministic, correctly inset, and absent where the material rules forbid them.
- [ ] Concave corners, convex corners, holes, steps, and chunk seams do not create obvious discontinuities.
- [ ] Moving or rotating a voxel body does not make its texture or macro variation slide.
- [ ] A reviewer can open the generated HTML gallery and compare all constructions and debug views without running the game or repositioning a camera.
- [ ] The result fits the concept work in `design/initial_researches` and `design/skyship-building-tool`.

### Outcome measurement

For each iteration, capture the same fixtures, camera, lighting presets, and metric set. Review results side by side, record the most important failure, implement one correction, and repeat. A material moves out of research only after the corrected version is accepted against both visual and performance criteria.

## 9. Release, observation, and correction cycles

### Cycle 1 — Wooden-plank surface mapping

- **Released:** Body-local face coordinates, 4×4-voxel wooden-plank mapping, renamed shipwright material, and an eight-fixture automated render gallery
- **Audience:** Internal development review
- **Expected result:** Connected faces read as coherent planks with stable direction and scale
- **Observation period:** Initial render on 2026-09-13, followed by one corrected render using the same fixtures and cameras
- **Measurements:** Eight fixtures from 1 to 53 voxels; greedy output from 24 to 132 vertices and 36 to 198 indices; visual review of broad faces, nominal x=16 seam, corners, holes, steps, hull shape, and transformed body. Mesh time, draw calls, and texture memory remain pending baseline-diagnostics work.
- **Feedback:** Body-local mapping stays continuous across greedy quads and does not move under an entity transform. A four-voxel texture footprint gives useful large-scale variation while plank seams preserve voxel-scale readability.
- **Unexpected effects:** The legacy texture contained baked nails, so macro mapping repeated vertical nail bands through face interiors. Side faces can become very dark under the single fixed lighting preset. The nominal x=16 fixture currently exercises coordinates across the boundary, not two independently meshed chunks.
- **Correction or decision:** A first correction removed the legacy nail bands. After review, the selected texture was revised again with clearer left/right bevels and restrained nails so repeated panels expose the gap between planks. It is now the canonical `wooden_plank.png`. Cycle 2 must still replace texture-bound placement with topology-aware borders where nails should follow the actual construction boundary. Keep the 4×4 scale for that experiment; address lighting through shading research rather than changing mapping.
- **Next review date:** Cycle 2 kickoff

### Cycle 2 — Wooden-plank borders and nails

- **Released:** Border-classified wooden plank with at least two nail techniques available for comparison
- **Audience:** Internal development and art review
- **Expected result:** Borders improve construction readability and nails appear only where structurally plausible
- **Observation period:** One focused comparison
- **Measurements:** Topology errors, geometry/draw-call delta, shimmer/bleeding, visual preference
- **Feedback:** Pending
- **Unexpected effects:** Pending
- **Correction or decision:** Select atlas/overlay or geometry/decal details and make one correction
- **Next review date:** Pending

### Cycle 3 — First contrasting material

- **Released:** Second material using the same generic contract
- **Audience:** Internal development and art review
- **Expected result:** Contract generalizes without wooden-plank conditionals
- **Observation period:** One implementation and correction cycle
- **Measurements:** Schema changes, renderer branches, asset effort, visual/performance criteria
- **Feedback:** Pending
- **Unexpected effects:** Pending
- **Correction or decision:** Stabilize, narrow, or redesign the material contract
- **Next review date:** Pending

## 10. Decision log

| Date | Decision | Evidence and rationale |
| --- | --- | --- |
| 2026-09-13 | Initiative planned from the voxel-shading thought | The current prototype proves editing and greedy meshing but not the target continuous surface language |
| 2026-09-13 | First implementation material is named wooden plank | It describes constructed boards and leaves natural wood available for bark/end-grain/organic materials |
| 2026-09-13 | Borders are a first-class material feature | Materials need distinct boundary treatments; wooden plank specifically needs optional nails |
| 2026-09-13 | Research includes mandatory correction cycles | Reference gathering or a first prototype does not establish a usable visual or technical decision |
| 2026-09-13 | Separate blocks, voxel materials, and modeled assets | Simulation identity, surface appearance, and authored geometry evolve independently and should share stable IDs rather than duplicate data |
| 2026-09-13 | Generate a deterministic visual-test gallery | Predefined constructions, fixed views, labeled PNGs, a contact sheet, and an HTML index make visual validation repeatable and easy to review |
| 2026-09-13 | Use a hybrid generative PBR workflow | AI is valuable for material concepts, but deterministic seamless processing, shared height-derived maps, semantic masks, Bevy channel packing, and relit validation are required for coherent production materials |
| 2026-09-13 | Pilot ComfyUI texture nodes; own validation and export | ComfyUI, TextureAlchemy, and Seamless-Texture cover generation and map processing, while project-owned tooling must enforce provenance, exact tiling, ORM layout, and voxel-specific render tests |
| 2026-09-14 | Treat generated borders as constrained geometry | Image generation supplies surface character, but deterministic per-band framing owns the exact outer bevel, seam rows, and removal of partial next-plank content |
| 2026-09-14 | Discover technical maps beside `base_color.png` | A conventional `normal.png` plus `orm.png` sibling set lets game materials adopt PBR when available and retain scalar fallbacks for legacy textures |

## 11. Closure

- **Final status:** Open
- **Closed on:** null
- **Completion results:** Pending
- **Resources used:** Pending
- **Reason for completing or discarding:** Pending
- **What we learned:** Pending
- **Follow-up records:** Pending
