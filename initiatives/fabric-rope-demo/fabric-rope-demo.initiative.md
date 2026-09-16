---
title: "Fabric and Rope Demo"
status: planned
owner: michael
created: 2026-09-13
updated: 2026-09-13
target_date: null
budget: null
related_thoughts:
  - thoughts/fabric-and-rope-system/fabric-and-rope-system.thought.md
related_stories: []
related_actors: []
tags:
  - aether-isles
  - fabric
  - rope
  - physics
  - xpbd
  - browser-game
  - demo
---

# Fabric and Rope Demo

## 1. Problem

- **Affected actors:** The Aether Isles development team, artists defining the voxel-built visual language, and future players whose ships and structures depend on readable flexible materials.
- **Current situation:** Concept sheets 14–20 describe fabric, rope, anchors, tension, pulleys, damage, water, and fire, but the project has no executable flexible-structure system. The existing `aether_voxels` feature stores and meshes rigid cells; it cannot establish whether low-resolution simulated sheets and chains will move convincingly, remain visually voxel-like, or run within the native and browser frame budgets.
- **Impact:** Fabric and rope affect sails, rigging, flags, canopies, and traversal. Building production features before validating their shared physical and visual core risks coupling Shipwright and other games to a solver or rendering treatment that does not satisfy the concept art.
- **Evidence:** The related [Fabric and Rope System thought](../../thoughts/fabric-and-rope-system/fabric-and-rope-system.thought.md) identifies a shared constraint graph as the leading hypothesis, but also records unresolved contradictions: globally axis-aligned tiles cannot form the pictured curves, true pulleys require more than fixed distance constraints, and balloons, trampolines, wetness, and fire each add separate mechanics. No running prototype or performance measurement currently resolves those questions.

The demo promise is:

> Manipulate a voxel-styled fabric panel and its supporting ropes, watch wind and tension reshape the structure, cut connections to produce failure, and compare rendering treatments in one repeatable scene.

This initiative creates evidence for a production decision. It does not claim to deliver the final ship-construction system.

## 2. Completion criteria

| Measure | Baseline | Target | Evidence source | Measurement window |
| --- | --- | --- | --- | --- |
| Runnable demo | No fabric or rope executable exists | A dedicated native app and `/game/fabric-rope-demo/` browser route load the same interactive scene | Native launch, web build, and browser smoke test | At demo release |
| Shared simulation | No flexible simulation crate exists | One reusable feature crate simulates both a 1D rope chain and a 2D fabric lattice through the same node/constraint contracts | Source dependency review and automated tests | Before demo release |
| Stable behavior | No solver baseline exists | The canonical scene runs for 60 simulated seconds without non-finite positions, runaway energy, or failed intact constraints exceeding the documented tolerance | Deterministic headless simulation test | Every implementation cycle |
| Force response | No wind or tension behavior exists | Fabric visibly responds to gravity, directional wind, and moving anchors; rope straightens under load and transmits anchor motion to the sheet | Scripted scenario capture and human acceptance check | Before demo release |
| Interactive failure | No flexible damage behavior exists | A user can cut a rope link and a fabric connection, see the topology change, and reset the exact scene without reloading | Browser interaction test and human acceptance check | Before demo release |
| Visual direction | Only concept images exist | Free local orientation, quantized orientation, and globally axis-aligned tile treatments can be compared from the same pose; one treatment is selected with rationale | Fixed-camera captures and decision log | Before production recommendation |
| Debuggability | No flexible diagnostics exist | The demo can display nodes, constraints, anchors, per-link strain, broken links, fixed-step count, node count, and frame/simulation timing | Debug-overlay acceptance check | Before performance measurement |
| Reference scale | No performance data exists | Native and WASM measurements are recorded for 100, 1,000, and 5,000 nodes; 1,000 nodes remain interactive on the documented reference machine/browser | Saved diagnostics and dated capture | Before initiative review |
| Production decision | No validated architecture exists | The initiative ends with an explicit adopt, revise, or reject decision for the constraint-graph approach and a bounded next integration step | Closing review against all measurements | At initiative close |

All required completion criteria:

- [ ] The demo is directly runnable on native and through the shared browser lobby.
- [ ] Rope and fabric use common simulation primitives without depending on demo-specific input or UI.
- [ ] Automated tests cover distance preservation, pinned anchors, deterministic stepping, breakage, component detection, and non-finite-state rejection.
- [ ] Wind, moving anchors, cutting, pausing, single-stepping, debug rendering, and reset are usable in the demo.
- [ ] Three voxel-tile rendering treatments are captured under identical simulation input and one is selected or all are rejected with reasons.
- [ ] Performance measurements include node count, constraint count, substeps, solver iterations, simulation time, and frame time.
- [ ] Known limitations and the production recommendation are recorded without presenting excluded mechanics as completed.

## 3. Constraints and stop conditions

### Constraints

- **Time:** Deliver the smallest evidence-producing demo before beginning Shipwright integration. Set a target date after the baseline solver milestone is estimated.
- **Budget:** No monetary budget is assigned. Use the existing Rust, Bevy, WASM, and repository build pipeline.
- **Capacity:** Keep one canonical scene and one reusable simulation crate. Avoid parallel production systems for individual use cases.
- **Technical or operational constraints:** Bevy 0.18.1, Rust 2024, native and `wasm32-unknown-unknown`; fixed-step simulation; deterministic constraint ordering; no required server or external service; the shared web host must remain functional.
- **Must preserve:** `aether_voxels` remains authoritative for rigid cell storage; flexible nodes are not inserted as moving voxel cells; game-specific controls and presentation do not enter the reusable solver; existing games and workspace tests continue to build.

### Stop or reconsider if

- [ ] The canonical intact scene still produces non-finite state or explosive instability after two materially different solver/timestep correction cycles.
- [ ] A 1,000-node scene cannot remain interactive in the reference WASM build after one profiling and optimization cycle.
- [ ] None of the three tile treatments reads as both flexible and voxel-built in the fixed-camera review.
- [ ] The shared graph requires rope- and fabric-specific data paths so divergent that the common abstraction adds more complexity than it removes.
- [ ] The MVP expands to require self-collision, per-voxel collision, full rigid-body coupling, balloons, water, fire, networking, or a production construction editor.
- [ ] Repeatable native and browser behavior cannot be obtained with the available fixed-step and numeric precision constraints.

When a stop condition is met, pause new features and preserve captures, metrics, and failing fixtures. Reduce the experiment to the smallest disputed hypothesis, revise the architecture, or reject the approach in the closing decision. Do not compensate for an unstable or visually weak core by adding more use cases.

## 4. Possible solutions

### Option A — Unified XPBD graph and dedicated demo app

- **Description:** Build an `aether_flexible` feature crate with nodes, distance constraints, compliance, fixed/tethered anchors, deterministic fixed stepping, cutting, and connected-component detection. Compose it in a dedicated `games/fabric_rope_demo` app with voxel-styled render reconstruction, controls, diagnostics, and benchmark presets.
- **Expected effect:** Directly validates the common fabric/rope hypothesis and produces reusable simulation code without coupling it to Shipwright.
- **Cost and effort:** Moderate. Requires solver code, rendering adapters, interaction, tests, native/browser registration, and measurement fixtures.
- **Risks:** XPBD tuning may hide energy artifacts; visual tiles may gap or overlap; rendering thousands of tile entities may dominate solver cost; deterministic behavior may differ across native and WASM floating point.
- **How it would be tested:** Deterministic solver tests, scripted canonical scenarios, fixed-camera render comparisons, browser smoke tests, and scale presets.

### Option B — Presentation-only animated mockup

- **Description:** Animate a fabric mesh and rope curves in a dedicated scene without a shared physical graph.
- **Expected effect:** Produces a fast visual reference and can help select tile styling.
- **Cost and effort:** Low to moderate.
- **Risks:** Cannot validate tension transfer, arbitrary anchors, cutting, solver stability, or reusable gameplay behavior. It may approve an appearance that the eventual simulation cannot reproduce.
- **How it would be tested:** Fixed-camera visual review only; it would not satisfy the physics completion criteria.

### Option C — Integrate directly into Shipwright

- **Description:** Add flexible pieces, construction controls, simulation, and rendering to the current ship editor as one vertical feature.
- **Expected effect:** Tests the system in its likely eventual context.
- **Cost and effort:** High, with simultaneous work on physics, construction UX, voxel-body attachment, persistence, and ship behavior.
- **Risks:** Failures become difficult to isolate and provisional contracts leak into production gameplay.
- **How it would be tested:** End-to-end Shipwright scenarios, after a smaller solver proof exists.

### Option D — Take no action

- **Likely consequence:** Fabric and rope remain concept-art promises; future sails, rigging, traversal, and damage either block or implement incompatible substitutes.
- **When this is the correct choice:** If no near-term experience depends on flexible structures or the voxel-tile direction is intentionally abandoned.

## 5. Selected solution

- **Decision:** Choose Option A: a unified XPBD graph in a reusable feature crate, exercised by a dedicated Demo app.
- **Why this option:** It addresses the highest-risk physics and visual assumptions in isolation while creating a credible reuse boundary. It also yields deterministic fixtures and performance evidence before any Shipwright-specific construction contract is selected.
- **Assumptions being made:** A CPU XPBD solver is adequate at the intended scale; low-resolution tiles can make gaps and deformation feel intentional; explicit sockets can later connect the graph to movable voxel bodies; fixed guide points are sufficient to represent pulley-like redirection in a later slice.
- **Known risks:** Solver parameters may be sensitive to step count; tile entities or mesh rebuilds may cost more than simulation; graph tearing can create many small components; WASM performance may require batching; visually rigid panels may expose discontinuities.
- **In scope:** Rope chains; masked rectangular fabric lattices; fixed and tethered anchors; distance and optional shear/bend constraints; gravity; directional wind; scripted anchor motion; pause and single-step; cutting; detached-component detection; reset; three tile-orientation treatments; tension/debug visualization; scale presets; native and browser delivery; tests and measurements.
- **Out of scope:** Shipwright placement and editing, save/load, final rigid-body force coupling, per-voxel collision, rope wrapping, true sliding pulleys and winches, self-collision, cloth-cloth collision, character interaction, trampolines, balloons or pressure constraints, wetness, fire propagation, network replication, production art, and performance guarantees beyond the documented reference scene.

## 6. Architecture and implementation plan

### Intended boundary

```text
crates/features/aether_flexible/
  topology, nodes, constraints, materials, fixed-step solver,
  breakage, connected components, diagnostics
                         │
                         ▼
games/fabric_rope_demo/
  canonical scene, input, camera, UI, scripted scenarios,
  voxel-style rendering, benchmark presets
                         │
                         ▼
shared native and browser app composition
```

The feature crate must not depend on `shipwright`. Its core simulation should avoid requiring Bevy rendering types even if Bevy math, ECS resources, or schedules host it. Rendering may initially live in the demo until the chosen tile treatment deserves a reusable presentation crate.

### Milestones

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Canonical scenario specification | Exact node layouts, anchors, forces, scripted motion, cut events, camera transforms, and presets are recorded | Related thought | pending | A fixture can be reconstructed without hand tuning |
| Flexible graph primitives | Stable node, constraint, material, topology, and anchor data types | Scenario specification | pending | Unit tests construct one rope and one sheet without presentation code |
| Fixed-step XPBD solver | Gravity, integration, distance constraints, compliance, damping, pins, and deterministic iteration | Graph primitives | pending | Stability, distance, anchor, determinism, and finite-state tests pass |
| Rope and fabric builders | A 16-link rope and masked 12×8 sheet compile into the common graph | Graph primitives | pending | Topology/count snapshot tests pass |
| Breakage and graph analysis | Selected constraints can break; detached components and anchor reachability are reported | Solver | pending | Table-driven cut scenarios return expected components |
| Native demo shell | Dedicated game composes the canonical scene, camera, controls, reset, pause, and single-step | Solver and builders | pending | `cargo run -p fabric_rope_demo --bin fabric_rope_demo_exec` opens a usable scene |
| Voxel-style rope renderer | Cuboid or authored modular links align between solved nodes and expose tension | Native shell | pending | Loaded and slack rope states are captured from fixed cameras |
| Three fabric render treatments | Free local orientation, quantized orientation, and globally axis-aligned tiles render from the same node pose | Native shell | pending | One input switches modes without resetting simulation; matched captures exist |
| Wind and scripted anchors | The sheet responds to constant directional wind and repeatable anchor motion | Solver and demo controls | pending | A scripted 60-second scenario completes and produces expected diagnostics |
| Interactive cutting | Pointer or debug-ray interaction selects and breaks rope/fabric constraints | Breakage and renderer | pending | Both cut cases change topology and reset exactly |
| Diagnostics and scale presets | Overlay and machine-readable output report topology, solver settings, strain, simulation time, and frame time for 100/1,000/5,000 nodes | Integrated scene | pending | Presets run without source edits and write comparable records |
| Browser registration | Demo is included in the shared WASM host, manifest, generated routes, and lobby | Integrated native demo | pending | `/game/fabric-rope-demo/` loads from `dist` and browser smoke test reaches ready state |
| Visual and performance review | Captures and native/WASM metrics are compared against completion and stop conditions | All demo features | pending | Rendering choice and adopt/revise/reject recommendation are recorded |

## 7. Test instructions

The exact package and script names below define the intended interface. Update this section and the decision log if implementation discovers a better stable name.

### Prerequisites

- Repository toolchain from `rust-toolchain.toml`
- `wasm32-unknown-unknown`
- `wasm-bindgen-cli` version matching `Cargo.lock`
- Node dependencies installed for browser smoke tests
- A browser supported by the existing Playwright configuration

### Automated verification

Run from the repository root:

```bash
cargo test -p aether_flexible
cargo test -p fabric_rope_demo
cargo clippy -p aether_flexible -p fabric_rope_demo --all-targets -- -D warnings
cargo build -p fabric_rope_demo --target wasm32-unknown-unknown
./scripts/build-web.sh
npm run test:browser
```

Expected result:

- Exit code: `0` for every command.
- Solver tests report no non-finite state and pass their documented constraint tolerances.
- The web manifest and built distribution include `fabric-rope-demo`.
- The browser smoke test can enter the demo and detect its ready marker without console errors.

The completed implementation should also provide one deterministic scenario command, for example:

```bash
cargo run -p fabric_rope_demo --bin fabric_rope_demo_exec -- \
  --scenario canonical --fixed-steps 3600 --report target/fabric-rope-demo/canonical.json
```

Expected result:

- Exit code: `0`.
- The JSON report records the fixture ID, node and constraint counts, step size, substeps, iterations, maximum strain, broken constraints, detached components, non-finite count, and simulation timing.
- `non_finite_count` is `0`.

### Human acceptance checks

- [ ] Michael confirms that rope reads as a tensioned constructed material rather than a smooth cable.
- [ ] Michael compares matched captures of all three fabric tile treatments and records a selection, requested revision, or rejection.
- [ ] With no external instructions, a reviewer can move an anchor, change wind, cut a rope, cut fabric, pause, single-step, toggle diagnostics, and reset.
- [ ] The canonical sheet visibly sags under gravity, fills under wind, follows anchor motion, and changes behavior after a cut.
- [ ] Debug visualization makes unstable tuning or overstrained links apparent before failure.

### Outcome measurement

Store dated native and WASM reports for 100, 1,000, and 5,000-node presets under a generated target directory, and summarize the stable results in this initiative. Capture identical camera frames for the three tile treatments. At review, compare those artifacts with every completion and stop condition, then record an adopt, revise, or reject decision. Generated benchmark artifacts need not be committed, but the selected captures and conclusions should be preserved in the initiative or a linked research record.

## 8. Release, observation, and correction cycles

### Cycle 1 — Core motion and rendering proof

- **Released:** Not yet released.
- **Audience:** Internal Aether Isles development review.
- **Expected result:** The canonical rope-and-sheet scene remains stable, communicates tension, and reveals a viable voxel-tile treatment.
- **Observation period:** One structured review after deterministic scenarios and matched captures are available.
- **Measurements:** Solver stability, maximum strain, native/WASM timing across presets, interaction reliability, and tile-treatment selection.
- **Feedback:** Pending.
- **Unexpected effects:** Pending.
- **Correction or decision:** Pending.
- **Next review date:** To be set when the integrated native demo is ready.

### Cycle 2 — Browser proof and production recommendation

- **Released:** Not yet released.
- **Audience:** Internal reviewers through the shared browser lobby.
- **Expected result:** The selected visual treatment and core interactions behave comparably in WASM and support a bounded production recommendation.
- **Observation period:** One correction cycle after Cycle 1.
- **Measurements:** Browser frame/simulation time, smoke-test reliability, visual comparison, and remaining architectural risks.
- **Feedback:** Pending.
- **Unexpected effects:** Pending.
- **Correction or decision:** Pending.
- **Next review date:** To be set after Cycle 1.

## 9. Decision log

| Date | Decision | Evidence and rationale |
| --- | --- | --- |
| 2026-09-13 | Initiative planned | The related thought identifies a concrete, bounded experiment and the user committed to building a Demo app |
| 2026-09-13 | Use a dedicated game and reusable feature crate | This isolates solver and visual risks while preserving a path into Shipwright and other games |
| 2026-09-13 | Select a unified XPBD graph for the first implementation | Rope and fabric share nodes, distance relationships, anchors, tension, and breakage; the demo will test whether the abstraction holds |
| 2026-09-13 | Compare three tile orientation treatments | The concept art's curved sheets conflict with its claim that tiles remain globally axis-aligned |
| 2026-09-13 | Exclude production integrations and secondary material reactions | The initiative exists to validate the common physical and visual core before breadth |

## 10. Closure

- **Final status:** Open — planned
- **Closed on:** Not closed
- **Completion results:** Pending
- **Resources used:** Pending
- **Reason for completing or discarding:** Pending
- **What we learned:** Pending
- **Follow-up records:** The expected follow-up is a bounded Shipwright integration initiative, a revised solver experiment, or a recorded rejection of the constraint-graph direction.
