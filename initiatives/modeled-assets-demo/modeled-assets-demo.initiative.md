---
title: "Modeled Assets Demo"
status: planned
owner: michael
created: 2026-09-13
updated: 2026-09-13
target_date: null
budget: null
related_thoughts:
  - thoughts/modeled-assets-in-voxel-grid/modeled-assets-in-voxel-grid.thought.md
related_stories: []
related_actors: []
tags:
  - aether-isles
  - voxels
  - modeled-assets
  - demo
  - rendering
---

# Modeled Assets Demo

## 1. Problem

- **Affected actors:** Developers validating the voxel engine, artists preparing authored models, and eventually players placing non-cube objects in voxel ships and islands.
- **Current situation:** `aether_voxels` stores one value per occupied cell and generates greedy cube meshes. The repository has no executable proof that an authored model can reserve one or several voxel cells, leave the correct opening in generated voxel geometry, move with its voxel body, and be referenced as one logical instance.
- **Impact:** Modeled blocks are already anticipated by the voxel-shading architecture, but their storage and rendering contract remains hypothetical. Building production assets before validating occupancy, anchors, rotations, removal, and mixed mesh boundaries risks spreading incompatible conventions through games and asset files.
- **Evidence:** The related modeled-assets thought identifies the mismatch between multi-cell grid occupancy and single-instance identity. The current `VoxelWorld<T>` contains only `blocks: HashMap<IVec3, T>`, while `greedy_mesh` assumes every opaque entry emits cube geometry.

The initiative will produce a small, standalone demo app that answers the architectural questions with visible, interactive evidence and deterministic tests. It is not a production asset pipeline.

## 2. Completion criteria

| Measure | Baseline | Target | Evidence source | Measurement window |
| --- | --- | --- | --- | --- |
| Single-cell modeled asset | Unsupported | One asset reserves one cell, renders once at the declared anchor, and can be placed and removed | Demo interaction and automated world tests | Demo acceptance run |
| Multi-cell modeled asset | Unsupported | One asset reserves at least two cells, supports at least four grid rotations, renders once, and can be removed from any footprint cell | Demo interaction and table-driven tests | Demo acceptance run |
| Voxel/model boundary | Model-backed cells would be cubes | Modeled-asset cells emit no cube geometry and adjacent voxel faces remain present | Mesh assertions and cutaway/debug view | Every test run |
| Spatial integrity | No asset occupancy index | All placed instances and occupied cells satisfy bidirectional index invariants after every tested edit sequence | Automated invariant tests | Every test run |
| Body transforms | No mixed body proof | Voxel mesh, asset models, footprints, and picking remain aligned after translating and rotating the parent voxel body | Fixed transformed fixture and human check | Demo acceptance run |
| Persistence | No modeled-asset world format | Demo state round-trips definition ID, instance ID or stable replacement, anchor, orientation, and required instance state | Serialization round-trip test | Every test run |
| Visibility experiment | Every model would be submitted | A surface asset renders and a model fully enclosed by opaque voxels can be hidden by a toggleable cached local-exposure rule | Automated exposure tests and demo counter/debug view | Demo acceptance run |
| Repeatable launch | No app | One documented command launches the demo and workspace checks remain green | Repository commands | Final implementation cycle |

All required completion criteria:

- [ ] A standalone demo app presents a one-cell asset, a rotated multi-cell asset, and an opaque enclosure containing an asset.
- [ ] Users can place and remove cube voxels and modeled assets without overlapping occupied cells or creating partial multi-cell instances.
- [ ] Removing any cell in an asset's footprint removes the entire logical instance and frees every owned cell.
- [ ] The renderer creates exactly one scene instance per visible logical asset, irrespective of footprint size.
- [ ] A debug mode displays anchors, occupied footprints, instance IDs, and exposure state.
- [ ] The generated voxel mesh leaves model-backed cells open and remains correct after edits.
- [ ] The demo distinguishes simulation occupancy, cube emission, and neighbor-face occlusion in its API.
- [ ] State can be saved and loaded without storing transient model paths, Bevy entities, or asset handles as world identity.
- [ ] Native tests, workspace checks, and the web build pass.
- [ ] Findings update the related thought, including whether exclusive cell occupancy is sufficient.

## 3. Constraints and stop conditions

### Constraints

- **Time:** No target date is assigned. Deliver the smallest demonstrable vertical slice before extending the system.
- **Budget:** No monetary budget is assigned. Use repository-owned geometry, a primitive generated in code, or an asset with compatible licensing.
- **Capacity:** One single-cell definition, one multi-cell definition, one enclosure fixture, and one voxel material are sufficient.
- **Technical or operational constraints:** Rust and Bevy; native execution is required and the existing WASM build must remain viable; voxel bodies use integer grid coordinates and discrete grid rotations; the shared voxel crate must not depend on Bevy presentation assets.
- **Must preserve:** Existing cube-voxel meshing behavior, deterministic world mutations, stable body-local coordinates, separation of simulation data from presentation handles, and unrelated work already present in the repository.

### Stop or reconsider if

- [ ] Supporting mixed occupancy requires replacing the current voxel storage and mesher rather than evolving their query boundary; pause and isolate a smaller storage prototype first.
- [ ] A model cannot remain aligned with the voxel mesh under parent translation and rotation after two transform corrections; redesign anchor and coordinate conventions before adding more assets.
- [ ] Multi-cell placement cannot maintain bidirectional index invariants through deterministic tests; do not add rendering or persistence until storage is correct.
- [ ] The visibility experiment couples simulation activity to render visibility; remove the optimization and ship the correct always-rendered path.
- [ ] The demo begins adding production concerns such as arbitrary rotation, animation, partial destruction, streaming, generalized attachments, or a broad asset editor before the required slice passes.
- [ ] The implementation needs game-specific conditionals in the shared voxel crate for either demo asset.

When a stop condition is met, retain the smallest passing storage or rendering experiment, record the failure in the decision log, and redesign or narrow the affected layer. Visibility optimization may be discarded without blocking the core demo.

## 4. Possible solutions

### Option A — Standalone vertical-slice demo on shared voxel APIs

- **Description:** Add a small demo game that consumes reusable occupancy, asset-instance, meshing-query, and exposure APIs from `aether_voxels`, with Bevy model spawning kept in the demo or an appropriate presentation crate.
- **Expected effect:** Validates the entire placement-to-render path in isolation while producing shared engine primitives usable by shipwright and later games.
- **Cost and effort:** Moderate. Requires storage evolution, tests, a small interactive scene, debug visualization, and model lifecycle handling.
- **Risks:** The shared API may be generalized too early from only two assets; changes can overlap voxel-shading work in the same crate.
- **How it would be tested:** Data-level invariants and mesh tests, fixed fixtures, interactive acceptance checks, serialization round trip, and native/web compilation.

### Option B — Prototype entirely inside shipwright

- **Description:** Add special block variants and spawn models directly from the existing shipwright world.
- **Expected effect:** Produces a visible result quickly with less initial API design.
- **Cost and effort:** Low initially, higher when extracting reusable behavior.
- **Risks:** Multi-cell occupancy, asset identity, and presentation handles become coupled to a game-specific `BlockKind`; existing building behavior becomes a moving test surface.
- **How it would be tested:** Manual placement in shipwright plus game-specific tests.

### Option C — Data-only prototype without a demo app

- **Description:** Implement occupancy and meshing tests but do not render or interact with modeled assets.
- **Expected effect:** Answers storage questions cheaply.
- **Cost and effort:** Low.
- **Risks:** Cannot validate anchors, model transforms, asset loading, visual seams, picking, body transforms, or visibility behavior.
- **How it would be tested:** Unit and integration tests only.

### Option D — Take no action

- **Likely consequence:** Modeled assets remain an unvalidated Phase 3 idea inside voxel shading. Future gameplay or art work will invent its own representation when the need becomes urgent.
- **When this is the correct choice:** If no planned Aether experience needs non-cube assets or if voxel storage is about to be replaced for independent reasons.

## 5. Selected solution

- **Decision:** Build Option A: a standalone vertical-slice demo backed by the smallest reusable voxel-world APIs.
- **Why this option:** The uncertainty spans storage, meshing, rendering, interaction, transforms, and visibility. Only an executable vertical slice tests their boundaries together without making shipwright responsible for an experimental representation.
- **Assumptions being made:** Exclusive whole-cell occupancy is adequate for the first slice; multi-cell assets are atomic; four yaw rotations are representative; adjacent voxel faces should remain rendered around model cells; local exposure is a useful conservative visibility signal; a simple authored or generated test model is enough to prove transforms.
- **Known risks:** Wall-mounted or embedded assets may require coexisting occupancy layers; multi-chunk ownership may complicate instance iteration; neighboring voxel faces can overdraw behind a model; model bounds may cross culling regions; visibility rules may misclassify sealed cavities with air gaps.
- **In scope:** Stable definition and instance IDs, explicit integer footprints, anchors, four yaw rotations, exclusive cell occupancy, atomic placement/removal, reverse indexing, meshing holes, adjacent face preservation, model spawning/despawning, parent voxel-body transforms, simple picking, debug visualization, save/load round trip, and toggleable local-exposure visibility.
- **Out of scope:** Production catalogs and import UI, arbitrary rotations, attachments sharing a voxel, animation, inventory, networking, partial damage, model voxelization, authored face-occlusion masks, exterior flood fills, GPU instancing, LODs, chunk streaming, final collision, polished art, and integration into shipwright.

## 6. Implementation plan

### Phase 0 — Fix the questions and fixtures

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Demo fixture specification | Exact one-cell, multi-cell, surface, enclosure, and transformed-body scenes are recorded | Related thought | pending | Each fixture declares cells, definition IDs, anchors, orientation, and expected exposure |
| Test asset definitions | One one-cell asset and one asymmetric multi-cell asset expose anchor/rotation errors | Fixture specification | pending | Definitions use stable IDs and explicit integer footprints; no runtime mesh bounds determine occupancy |
| Coordinate convention | Cell center, anchor, model offset, rotation pivot, and voxel-body transform are documented | Test definitions | pending | Expected transforms for four orientations are table-tested |

### Phase 1 — World representation

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Identity types | Definition and instance IDs cannot be confused with paths, handles, or each other | Coordinate convention | pending | Type-level usage and equality/serialization tests pass |
| Occupancy representation | Cube voxels and modeled-asset back-references answer one authoritative cell query | Identity types | pending | Occupied/free queries pass for every footprint cell |
| Atomic mutation API | `can_place_asset`, placement, voxel placement, and removal preserve invariants | Occupancy representation | pending | Table-driven success, overlap, rollback, and remove-from-any-cell tests pass |
| Index validation | Debug/test validation detects dangling cells, missing cells, overlap, and orphan instances | Mutation API | pending | Invalid synthetic worlds produce actionable errors |
| Persistence slice | Canonical instances and voxels round-trip; reverse occupancy is rebuilt or validated | Stable representation | pending | Round-trip produces an equivalent valid world without presentation data |

### Phase 2 — Mixed meshing

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Cell render semantics | Meshing queries distinguish occupied, emits-cube, and occludes-face | Occupancy representation | pending | Unit tests cover cube, air, and modeled-asset cells independently |
| Modeled-asset holes | No cube geometry is emitted for asset footprints | Cell render semantics | pending | Expected face/vertex/index counts pass for single- and multi-cell fixtures |
| Conservative boundaries | Neighboring cube faces remain present beside model cells | Cell render semantics | pending | Boundary-face assertions and cutaway render agree |
| Dirty-region selection | Edits invalidate footprint chunks and affected neighbors once | Mutation API | pending | Boundary fixtures produce the expected deduplicated dirty set |

### Phase 3 — Demo rendering and interaction

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Standalone demo crate | One documented command launches a focused modeled-assets scene | Fixtures, mixed meshing | pending | App starts without modifying another game's state |
| Definition-driven loading | Stable IDs resolve cached model scenes and authored offsets | Test asset definitions | pending | Missing IDs fail visibly; repeated definitions reuse loaded assets |
| Instance reconciliation | Exactly one model entity exists per canonical placed asset | Loading, world representation | pending | Place, rotate, remove, reload, and reset leave no duplicates or orphans |
| Grid interaction | User can select, place, rotate, and remove cube and model-backed content | Instance reconciliation | pending | All actions work from any relevant footprint cell and reject overlap visibly |
| Body-transform fixture | Mixed geometry stays aligned on a translated and rotated voxel-body parent | Coordinate convention | pending | Fixed view plus picking succeeds before and after body transform |
| Debug overlay | Anchors, footprints, IDs, dirty regions, and exposure state can be inspected | Rendering and interaction | pending | One toggle displays all required diagnostics legibly |

### Phase 4 — Visibility experiment and conclusion

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Local exposure classifier | An instance is exposed when at least one footprint boundary touches a non-occluding cell | Cell semantics | pending | Enclosed, surface, multi-cell, and edit-transition tests pass |
| Cached invalidation | Exposure changes only when relevant neighboring occupancy changes | Classifier, mutation events | pending | Enclose/reveal sequences update without a full-world rescan |
| Visibility toggle and counters | Demo compares always-rendered and exposure-filtered modes | Classifier, rendering | pending | UI reports total, exposed, and spawned model counts matching fixtures |
| Acceptance capture | Screenshots or gallery views preserve normal, footprint, cutaway, transformed, and visibility states | Complete demo | pending | Labeled evidence exists for every required fixture |
| Findings and API review | Related thought records evidence, contradictions, and the exclusive-occupancy decision | All phases | pending | Decision log links results and identifies the next production step |

## 7. Test instructions

### Prerequisites

- Stable Rust toolchain specified by the repository
- Native GPU-capable environment for interactive acceptance
- `wasm32-unknown-unknown` and existing web-build prerequisites for browser compatibility
- The test models or generated primitives committed with compatible provenance

### Automated verification

The final crate name may be adjusted to match workspace naming, but record any change here and keep one canonical launch command. Run from the repository root:

```bash
cargo fmt --all -- --check
cargo test -p aether_voxels
cargo test -p aether_modeled_assets_demo
cargo check --workspace
./scripts/build-web.sh
```

Required automated cases:

- place one-cell and asymmetric multi-cell assets in each supported orientation;
- reject any placement with one or more occupied footprint cells without partial mutation;
- remove a multi-cell instance by selecting each of its footprint cells;
- validate canonical-instance/reverse-index correspondence after edit sequences;
- serialize, load, rebuild the index, and compare equivalent state;
- verify cube emission and adjacent face counts around modeled cells;
- verify affected chunk/region selection for footprints crossing a boundary;
- classify exposed, enclosed, and newly revealed instances;
- reconcile render descriptors or entities without duplicates across unchanged revisions.

Expected result:

- Exit code: `0`
- Expected output or generated artifact: All invariant, meshing, transform, persistence, exposure, workspace, and web-build checks pass. No test relies on filesystem iteration order or transient Bevy entity IDs.

### Human acceptance checks

- [ ] Launching the documented command opens directly into the demo with concise controls visible.
- [ ] The one-cell asset is centered and oriented according to its definition.
- [ ] The asymmetric multi-cell asset visibly rotates around the declared anchor and its footprint follows it.
- [ ] Placement previews distinguish valid and blocked footprints before mutation.
- [ ] Removing any occupied part of a multi-cell asset removes one whole model and frees the complete footprint.
- [ ] Cube geometry does not pass through the visible model, and no unintended empty crack appears at mixed boundaries.
- [ ] Debug overlays make anchor and footprint mistakes apparent without reading logs.
- [ ] Voxel mesh and modeled assets remain aligned when their shared body translates and rotates.
- [ ] With exposure filtering enabled, the enclosed asset is absent while a surface asset remains visible; disabling it restores all assets.
- [ ] Editing the enclosure reveals and hides the contained asset without restarting the demo.

### Outcome measurement

This initiative succeeds by reducing architectural uncertainty, not by player adoption. During final review, record each completion criterion as pass, fail, or deferred with links to tests and captures. Also record:

- whether exclusive occupancy supports every demo interaction;
- how many shared APIs required demo-asset-specific branches, with a target of zero;
- whether the visibility optimization changed correctness or simulation behavior, with a target of no;
- any storage or mesh-generation regressions found in existing voxel tests.

If the core placement and rendering criteria pass but local-exposure visibility fails, complete the core demo and return visibility to the thought as a separate optimization question.

## 8. Release, observation, and correction cycles

### Cycle 1 — Storage and mixed-mesh proof

- **Released:** Headless fixtures for single-cell and multi-cell placement, reverse indexing, removal, and mesh holes
- **Audience:** Internal engine review
- **Expected result:** The representation maintains one logical asset and correct occupancy/geometry for all deterministic edits
- **Observation period:** One implementation pass followed by one invariant and API review
- **Measurements:** Passing cases, rollback failures, face counts, dirty-region sets, and number of asset-specific shared branches
- **Feedback:** Pending
- **Unexpected effects:** Pending
- **Correction or decision:** Correct the representation before proceeding to interactive rendering
- **Next review date:** After Phase 2

### Cycle 2 — Interactive transform and boundary proof

- **Released:** Standalone app with placement, rotation, removal, model reconciliation, parent-body transform, and debug overlays
- **Audience:** Internal engine and art review
- **Expected result:** Authored models align with their declared footprints and coexist visually with generated voxel geometry
- **Observation period:** One focused acceptance session and one correction pass
- **Measurements:** Completion checklist, anchor/rotation failures, duplicate/orphan entities, and mixed-boundary defects
- **Feedback:** Pending
- **Unexpected effects:** Pending
- **Correction or decision:** Stabilize coordinate and lifecycle contracts before starting visibility work
- **Next review date:** After Phase 3

### Cycle 3 — Enclosed-asset visibility experiment

- **Released:** Toggleable local-exposure filtering, counters, and an editable opaque enclosure
- **Audience:** Internal engine review
- **Expected result:** Fully surrounded models are skipped without affecting exposed models or simulation state
- **Observation period:** One experiment and correction pass
- **Measurements:** Total/exposed/spawned counts, invalidation correctness, classification time, and edit behavior
- **Feedback:** Pending
- **Unexpected effects:** Pending
- **Correction or decision:** Keep, simplify, defer, or reject the optimization independently of the core demo
- **Next review date:** Initiative completion review

## 9. Decision log

| Date | Decision | Evidence and rationale |
| --- | --- | --- |
| 2026-09-13 | Initiative planned from the modeled-assets thought | The current engine has no executable evidence for single- or multi-cell model-backed content |
| 2026-09-13 | Use a standalone demo rather than shipwright | The representation can be tested without coupling experimental storage to an existing game's private block model |
| 2026-09-13 | Build reusable storage semantics but keep presentation local | Occupancy and meshing belong to the voxel engine; model handles and Bevy scene lifecycle do not |
| 2026-09-13 | Preserve adjacent voxel faces around model cells in the first slice | Conservative geometry avoids cracks where authored meshes do not seal cell boundaries |
| 2026-09-13 | Make visibility optional and independently shippable | Correct world identity and rendering take priority over an unmeasured optimization |
| 2026-09-13 | Test an asymmetric multi-cell footprint | Symmetric assets can conceal incorrect anchor and rotation math |

## 10. Closure

- **Final status:** Open
- **Closed on:** null
- **Completion results:** Pending
- **Resources used:** Pending
- **Reason for completing or discarding:** Pending
- **What we learned:** Pending
- **Follow-up records:** Pending
