---
title: "Skyship Building Tool"
status: evaluating
owner: michael
created: 2026-08-25
updated: 2026-08-25
target_date: null
budget: null
related_thoughts:
  - thoughts/first-public-experience/first-public-experience.thought.md
related_stories: []
related_actors: []
tags:
  - aether-isles
  - skyship
  - building
  - creative-tool
  - browser
  - public-release
---

# Skyship Building Tool

## 1. Problem

- **Affected actors:** People attracted to building, voxel or modular art, fantasy vehicles, and shareable creative tools; development teams that need evidence about ship construction interaction and visual language.
- **Current situation:** The project has no way to assemble, save, present, or share a skyship. The full game's voxel construction and emergent physics are large unresolved systems, so waiting for them would delay learning whether building skyships is appealing on its own.
- **Impact:** A focused browser builder can give *Aether Isles* a public creative experience early, produce images and designs worth sharing, and inform later construction and ship-component systems without requiring survival, navigation, or a complete voxel engine.
- **Evidence:** The first-public-experience thought identified a skyship builder and gallery as a promising creation-and-sharing direction. No external usage evidence exists yet.

The experience promise is:

> Assemble a distinctive little skyship, pose it in the clouds, and share what you made.

## 2. Completion criteria

| Measure | Baseline | Target | Evidence source | Measurement window |
| --- | --- | --- | --- | --- |
| Publicly accessible tool | No builder exists | One stable browser URL opens the builder and a new design | Deployed build and release record | At release |
| First creation | No workflow exists | Five first-use testers place parts and produce a recognizable ship without external instruction | Observed sessions recorded as stories | First two weeks after release |
| Core editing | No construction model | Add, select, rotate, move where allowed, recolor where allowed, duplicate, and delete are usable with undo/redo | Automated state tests and acceptance checklist | Before release |
| Persistence | No design format | A design survives save/export and reload without structural or visual changes | Round-trip tests | Before release |
| Shareability | No output | Users can export a clean image and a shareable design artifact or link | Browser test and manual verification | Before release |
| Creative range | No catalog | The initial catalog can produce at least ten visibly distinct ships in an internal design exercise | Saved design fixtures and contact sheet | Before release |
| Browser reliability | No builder loop | Five consecutive create-save-reload-export sessions complete without crash or lost edits | Automated and human session | Before release |
| Reusable architecture | No building domain | Ship design model, presentation, UI, and optional voxel/physics adapters are separate libraries | Dependency audit | Before release |

All required completion criteria:

- [ ] A visitor can create, inspect, save, reload, and share a skyship from the browser.
- [ ] The first release provides a deliberately small but expressive parts and color catalog.
- [ ] Design data is independent of Bevy entities and can be serialized and tested headlessly.
- [ ] The tool does not require the production voxel engine, physics simulation, multiplayer, or 3D-print export.
- [ ] First-use observations and shared creations are captured as stories or evidence.

## 3. Constraints and stop conditions

### Constraints

- **Time:** Reach a polished, narrow creation loop in weeks. Set a target date after testing selection, placement, and browser export spikes.
- **Budget:** No monetary budget is currently assigned. Use original or properly licensed visual assets.
- **Capacity:** Multiple agent teams may work on the domain model, editing interaction, presentation, and sharing in parallel after the design contract is fixed.
- **Technical or operational constraints:** Browser-first, native-capable, keyboard and mouse initially, stable serialized format, no server dependency required for the smallest version.
- **Must preserve:** Ship designs must not be encoded only as live Bevy entities. Future voxel, simulation, collaboration, or export systems must be able to consume the design through explicit adapters.

### Stop or reconsider if

- [ ] First-use testers cannot place and correct parts without assistance after two interaction revisions.
- [ ] The initial catalog cannot produce visibly distinct ships without requiring a large asset-production effort.
- [ ] Browser image export or design persistence is unreliable after one focused correction cycle.
- [ ] Scope expands to require accounts, a hosted gallery, collaborative editing, full voxel simulation, or production-quality 3D-print output for the first release.
- [ ] Shared images do not visibly communicate *Aether Isles* after two presentation passes.

When a condition is met, reduce the catalog or sharing model, redesign the interaction, or pause the initiative. Do not compensate for weak building interaction by adding more parts.

## 4. Possible solutions

### Option A — Socketed modular-part builder

- **Description:** Users place authored hull, deck, mast, sail, balloon, engine, tank, and decorative modules through compatible sockets and grid-aware transforms.
- **Expected effect:** Produces attractive ships quickly with a manageable implementation and asset scope.
- **Cost and effort:** Requires a small authored parts kit, socket metadata, placement rules, and polished selection tools.
- **Risks:** May feel like assembling presets rather than genuinely building.
- **How it would be tested:** Give a small kit to five users and compare the diversity and completion time of their designs.

### Option B — Free voxel ship builder

- **Description:** Users build hulls and components voxel by voxel using the same conceptual representation intended for the game.
- **Expected effect:** Offers broad creative freedom and closer technical reuse with the final construction fantasy.
- **Cost and effort:** Requires voxel storage, meshing, editing, functional-block representation, selection, and stronger performance work.
- **Risks:** Delays public release and makes attractive results harder for first-time users.
- **How it would be tested:** Build a constrained voxel editing spike and compare time-to-first-ship against the modular approach.

### Option C — Take no action

- **Likely consequence:** The project loses an early creation-focused public artifact and learns about ship-building UX only after more of the game exists.
- **When this is the correct choice:** If the movement experience alone consumes available capacity or visual asset production prevents a credible builder.

## 5. Selected solution

- **Decision:** No implementation solution selected yet. Socketed modular parts and free voxel construction must be compared through constrained interaction and creative-range experiments. Both should target a neutral, versioned design contract where feasible.
- **Why this option:** Pending evidence about time to first satisfying ship, design diversity, technical cost, and transfer to the main game.
- **Assumptions being made:** Users value visual creation without physical validation; image and design sharing are sufficient for the first release; a representation-independent portion of the design model is possible.
- **Known risks:** Modular designs may look repetitive; voxel editing may delay release; sockets can frustrate placement; browser screenshots and downloads require platform-specific handling.
- **In scope:** To be finalized after comparing the two construction spikes. The evaluated scope currently includes an orbit camera, part or block catalog, selection, placement, rotation, deletion, duplication, visual variants, undo/redo, save/load, image export, design sharing, atmospheric presentation, and native/WASM builds.
- **Out of scope:** The current boundary excludes accounts, a public hosted gallery, moderation, real-time collaboration, physical flight validation, structural destruction, manufacturing guarantees, and production-ready 3D-print export. This boundary will be confirmed with the selected solution.

## Design architecture

The authoritative design must be plain serializable data:

```text
ShipDesign
  ├── design metadata
  ├── part instances
  │     ├── stable part definition ID
  │     ├── parent or attachment socket
  │     ├── local transform
  │     └── visual variants
  └── format version
```

Bevy entities are a projection of `ShipDesign`, not the saved design itself:

```text
ShipDesign ──> presentation adapter ──> Bevy mesh/material entities
     │
     ├──> file/link serializer
     ├──> image presentation
     ├──> future voxel conversion
     ├──> future physics validation
     └──> future 3D export
```

Editing operations should be commands with reversible effects:

```text
AddPart
RemovePart
MovePart
AttachPart
RotatePart
SetVariant
```

This gives undo/redo, deterministic tests, replayable edits, and a future collaboration seam without adding networking now.

## Dependency definition

### Dependency direction

```text
Bevy rendering/assets/input/window
             │
     ship design model
             │
   parts catalog + attachment rules
             │
    editing commands + history
             │
    ┌────────┼───────────┐
    │        │           │
selection  scene view  serialization
    │        │           │
    └────────┼───────────┘
             │
 image/design sharing + builder UI
             │
      skyship builder app
```

### Required dependencies

| Dependency | Provides | Must not know about |
| --- | --- | --- |
| Bevy renderer and assets | Static meshes, materials, textures, lighting, cameras, visibility | Ship-design rules |
| `aether_ship_design` | Stable serializable design, part instances, attachment references, format version | Bevy entities, UI, filesystem or browser APIs |
| `aether_part_catalog` | Part definitions, sockets, compatibility, bounds, visual variants | Current design, UI state |
| `aether_build_commands` | Validated reversible edits and history | Input devices, rendering |
| Selection/picking | Screen/world selection and gizmo targets | Design serialization, part semantics beyond metadata |
| `aether_ship_presenter` | Projects design instances into Bevy meshes and materials | Editing history, sharing transport |
| Camera system | Orbit, pan, zoom, framing, presentation poses | Ship data internals |
| Serialization | Versioned encode/decode and round-trip validation | Bevy scene lifecycle |
| Platform sharing adapter | Browser download, clipboard/link support, image capture | Ship editing rules |
| Builder UI | Catalog browsing, selected-part controls, undo/redo, export actions | Renderer and serializer internals |

### Optional and later dependencies

- A voxel adapter may convert voxel bodies or voxel-oriented designs into the neutral design/presentation pipeline.
- Physics may consume generated collision geometry and mass properties to estimate whether a design can fly.
- A 3D export adapter may generate glTF, STL, or another format after export requirements are defined.
- A hosted service may store designs and galleries after local or encoded sharing is validated.
- Multiplayer or collaboration may exchange edit commands after authoritative and conflict-resolution models are defined.

The initial builder must not depend on the voxel engine. The design contract must not prevent a future voxel-backed catalog.

## Shared dependencies with Rope, Glide, and Run

The two initiatives may share infrastructure without coupling their experiences:

- Bevy app configuration and web packaging;
- lobby and game catalog;
- browser smoke-test infrastructure;
- camera utilities where behavior overlaps;
- input-action mapping conventions;
- asset and material conventions;
- atmospheric sky, clouds, color palette, lighting, and presentation assets;
- diagnostics and performance reporting;
- release and media-capture workflow.

The builder must not depend on actor, rope, glide, runner, or scoring crates. The movement game must not depend on ship-design or builder UI crates.

## 6. Implementation plan

The plan is intentionally parallel after the design and catalog contracts are drafted.

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Define design contract | Versioned `ShipDesign`, instances, attachment references, and validation rules | None | pending | Headless fixtures serialize and round-trip |
| Define initial experience card | First-minute flow, share moment, smallest credible release, and visual target | First-public-experience thought | pending | Reviewed experience card stored or linked |
| Parts-kit exploration | Small original kit capable of varied silhouettes | Design contract draft, art direction | pending | Ten internal designs assembled and shown in one contact sheet |
| Placement interaction spike | Select, preview, attach, rotate, delete, and correct mistakes | Design and catalog drafts, picking | pending | First-use observation on primitive parts completed |
| Editing command history | Reversible validated operations with undo/redo | Design contract | pending | Deterministic command and inverse-command tests pass |
| Design presentation | Bevy entities track design changes without becoming authoritative | Design contract, assets | pending | Rebuild and incremental update match saved fixtures |
| Camera and scene presentation | Orbit, pan, zoom, auto-frame, lighting, cloud backdrop | Bevy renderer | pending | Works with smallest and largest reference fixtures |
| Save/load | Browser and native round-trip of versioned design artifact | Serialization, platform adapter | pending | Automated round-trip test and browser download test pass |
| Image export | Clean composition without editor UI | Presentation, platform adapter | pending | Exported image matches chosen viewport and contains no controls |
| Shareable design | Local file or encoded link opens the same design | Serialization, browser routing | pending | Cross-session/browser test restores fixture |
| Builder UI polish | Catalog, selection, actions, errors, undo/redo, export | Editing and presentation | pending | Five first-use sessions measured |
| Browser optimization | Accepted load and frame behavior for reference designs | Integrated builder | pending | Browser diagnostics and smoke tests pass |
| Public release | Stable URL, media, and observation plan | Completion criteria | pending | External URL and release record verified |

### Parallel workstreams

After the design contract is drafted, these can proceed concurrently:

- initial parts kit and visual language;
- placement and selection interaction;
- edit-command history;
- presentation and camera scene;
- serialization and platform export;
- builder UI sketches;
- browser diagnostics and release tooling.

## 7. Test instructions

### Prerequisites

- Stable Rust toolchain with `wasm32-unknown-unknown`
- `wasm-bindgen-cli` version matching `Cargo.lock`
- Node.js dependencies installed with `npm install`

### Automated verification

Run from the repository root:

```bash
cargo fmt --all -- --check
cargo check --workspace
./scripts/build-web.sh
npm run test:browser
```

Expected result:

- Exit code: `0`
- Expected output or generated artifact: Workspace checks pass, builder artifacts are generated, and browser smoke tests complete.

Add deterministic tests for design validation, edit operations, undo/redo, format migration, round-trip persistence, and reference-design rendering as their crates are introduced.

### Human acceptance checks

- [ ] A first-time user understands how to place the first part.
- [ ] Valid and invalid attachment locations are visually distinct.
- [ ] Correcting a mistake is immediate and safe.
- [ ] Camera controls do not interfere with placement controls.
- [ ] A small catalog produces visibly different silhouettes.
- [ ] The ship remains readable against the atmospheric background.
- [ ] Exported images look intentional enough to share without editing.
- [ ] Save and share language makes it clear what leaves the browser and what remains local.

### Outcome measurement

Observe at least five first-use creation sessions. Record time to first placed part, first correction, completed design, export attempt, points of confusion, catalog requests, and whether the participant wants to share the result. Preserve representative designs and record observations as stories.

## 8. Release, observation, and correction cycles

### Cycle 1 — Date to be selected

- **Released:** Constrained browser builder with local save and image export
- **Audience:** Internal testers followed by a small external group interested in creative tools
- **Expected result:** Testers complete distinct ships, correct mistakes without help, and choose to export or retain the result
- **Observation period:** Five observed sessions or one week
- **Measurements:** To be recorded
- **Feedback:** To be recorded as stories
- **Unexpected effects:** To be recorded
- **Correction or decision:** Refine modular builder, explore voxel editing, add sharing, or stop
- **Next review date:** To be selected when the first usable build is released

## 9. Decision log

| Date | Decision | Evidence and rationale |
| --- | --- | --- |
| 2026-08-25 | Initiative created in `evaluating` status | The creation experience is selected, but representation and interaction require validation |
| 2026-08-25 | Begin with a neutral ship-design model | Saved designs must not be coupled to Bevy entities, UI, physics, or voxel storage |
| 2026-08-25 | Compare socketed modular parts with constrained voxel construction | The shortest route to attractive results and the value of direct voxel reuse are not yet supported by evidence |
| 2026-08-25 | Keep voxel and physics integration optional | The first experience is creation and sharing, not proof of physical flight |
| 2026-08-25 | Exclude production 3D-print export from the first release | Manufacturing requirements are substantial and have not been defined |

## 10. Closure

- **Final status:** Open
- **Closed on:** Not closed
- **Completion results:** Not yet measured
- **Resources used:** Not yet recorded
- **Reason for completing or discarding:** Not applicable
- **What we learned:** Pending experiments and release
- **Follow-up records:** Pending
