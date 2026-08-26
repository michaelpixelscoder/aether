---
title: "Rope, Glide, and Run"
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
  - movement
  - rope
  - glider
  - browser-game
  - public-release
---

# Rope, Glide, and Run

## 1. Problem

- **Affected actors:** Early players and viewers of *Aether Isles*, and the development teams that need a concrete movement experience against which to evaluate reusable actor, rope, and gliding systems.
- **Current situation:** The project has a basic first-person camera that can translate freely over a grid. It does not yet provide grounded locomotion, a physical character body, running or jumping, rope swinging, gliding, an obstacle course, or a coherent short game.
- **Impact:** The full game depends heavily on enjoyable three-dimensional movement, but isolated movement features cannot establish whether the experience is readable, expressive, or worth sharing. A compact runner can put movement into public hands early while producing reusable actor and traversal systems.
- **Evidence:** The first-public-experience thought identified glider descent and rope-run escape as promising, immediately understandable directions. No external playtest evidence exists yet; generating that evidence is part of this initiative.

The experience promise is:

> Run through a collapsing route in the sky, grapple anchors to preserve momentum, and deploy a glider to cross impossible gaps.

The intended form is inspired by the immediacy and replayability of an endless runner such as *Temple Run*, but its movement, routes, visual identity, and traversal decisions must be original to *Aether Isles*.

## 2. Completion criteria

| Measure | Baseline | Target | Evidence source | Measurement window |
| --- | --- | --- | --- | --- |
| Publicly accessible experience | No movement game exists | One stable browser URL starts a complete playable run | Deployed build and release record | At release |
| Time to meaningful action | Only free camera movement exists | A new visitor can begin running, jumping, grappling, or gliding within 30 seconds without external instructions | Five observed first-use sessions | First two weeks after release |
| Traversal vocabulary | Walk/fly camera only | Running, jumping, rope swinging, rope release, glider deployment, gliding, landing, failure, and restart are usable in one run | Acceptance checklist and recorded run | Before release |
| Mechanical usefulness | No evidence | At least three of five observed players deliberately use both rope and glide to improve or complete a run | Playtest notes recorded as stories | First two weeks after release |
| Reliability | No game loop | Five consecutive complete start-to-failure-to-restart cycles run without crash, stuck state, or reload | Browser smoke test and human session | Before release |
| Browser support | Current sandbox smoke-tested in one browser | The release passes the agreed browser matrix and records average frame time for the reference route | Automated tests and diagnostics capture | Before release |
| Shareable output | No clips from the experience | At least three ten-to-thirty-second clips clearly communicate running, rope swinging, and gliding | Project media folder or release record | Before public announcement |
| Reusable architecture | Basic controller is coupled to the camera entity | Locomotion, rope, and glide are independently enabled plugins operating through shared actor/control contracts | Dependency audit and lab builds | Before release |

All required completion criteria:

- [ ] A complete run has a start, escalating traversal, failure or finish, score or time feedback, and immediate restart.
- [ ] The browser release includes grounded running, jumping, at least one satisfying rope swing, and at least one meaningful glide.
- [ ] Locomotion, rope, and glide each run in an isolated lab using the same library used by the game.
- [ ] The game records enough diagnostics to distinguish game-design problems from frame-rate or simulation failures.
- [ ] First-use observations and public reactions are captured as stories.

## 3. Constraints and stop conditions

### Constraints

- **Time:** The first public version should be scoped in weeks, not allowed to grow into a miniature survival game. A target date will be selected after the first movement and physics spikes.
- **Budget:** No monetary budget is currently assigned. Prefer Bevy and permissively licensed Rust dependencies that support native and WASM builds.
- **Capacity:** Multiple AI-agent teams will work in parallel. Shared contracts must be documented before teams integrate against them.
- **Technical or operational constraints:** Native and browser builds are mandatory. The browser is the primary distribution path. Simulation must tolerate browser timing and performance limits.
- **Must preserve:** Movement systems must remain usable outside this runner. Rope and glide must not depend on runner-specific scoring, route generation, or the voxel engine.

### Stop or reconsider if

- [ ] No tested physics approach can produce stable grounded motion and rope constraints in the target browser matrix.
- [ ] Rope swinging remains difficult to understand or control after three documented tuning experiments with materially different control models.
- [ ] Gliding adds no meaningful route choice after two distinct route-design experiments.
- [ ] The smallest credible experience grows to require voxel destruction, procedural worlds, combat, inventory, or multiplayer for its first release.
- [ ] Performance on the reference browser/hardware baseline remains below the selected target after profiling and one focused optimization cycle.
- [ ] First-use tests show that players cannot discover the core traversal loop within 30 seconds after two onboarding revisions.

When a stop condition is met, pause the affected direction, record the result as a story, and choose among reducing scope, replacing the control model, or discarding the initiative. Do not hide an unresolved movement problem behind more content.

## 4. Possible solutions

### Option A — Authored time-trial course

- **Description:** A short handcrafted route with deliberate running, rope, and glide challenges. Players optimize time, flow, and collectible routing.
- **Expected effect:** Gives precise control over teaching and tuning each movement mechanic and is the shortest route to a polished experience.
- **Cost and effort:** Requires authored level pieces and several route iterations, but little procedural generation.
- **Risks:** Limited replayability unless movement depth, alternate routes, scoring, or daily variations are strong.
- **How it would be tested:** Publish one greybox course, observe first-use discovery, and measure completion and restart behavior.

### Option B — Endless or generated runner

- **Description:** Route segments are selected or generated continuously as speed and difficulty increase.
- **Expected effect:** Creates immediate replayability and score competition closer to the runner reference.
- **Cost and effort:** Requires safe segment assembly, difficulty pacing, fairness validation, and more failure handling.
- **Risks:** Procedural work may distract from movement feel and produce generic or unfair routes.
- **How it would be tested:** Build a segment grammar using a small authored set and measure invalid routes and repetition before expanding it.

### Option C — Take no action

- **Likely consequence:** Movement remains an internal technical concern, delaying public feedback about one of the game's central experiences.
- **When this is the correct choice:** If early physics spikes show that acceptable rope or character behavior is not feasible in the browser within the intended scope.

## 5. Selected solution

- **Decision:** No implementation solution selected yet. The committed direction is a short movement experience combining running, rope swinging, and gliding. Authored-course and endless-runner structures remain under evaluation.
- **Why this option:** Pending comparison of the movement greybox and route-structure options.
- **Assumptions being made:** Running, swinging, and gliding can form a coherent movement flow; a predefined character body is sufficient; the first version does not require the voxel engine.
- **Known risks:** Physics tuning may be slow; rope control can feel chaotic; gliding can trivialize routes; fast movement can be uncomfortable in first person; browser input and frame timing may change feel.
- **In scope:** To be finalized after the greybox experiments. The evaluated scope currently includes actor architecture, player control, grounded locomotion, custom movement extension points, rope, glider, camera, a runner route, obstacles, scoring or timing, restart, diagnostics, native/WASM builds, and a public browser release.
- **Out of scope:** The current scope boundary excludes voxel construction and destruction, survival, combat, inventory, NPC gameplay, production multiplayer, procedural islands, ships, Aether networks, and a full open world. This boundary will be confirmed with the selected solution.

## Actor and movement architecture

### Runner lane controls

The runner uses multiple fixed lanes rather than unrestricted lateral movement. The
local controller translates device-specific input into a semantic lane-change
intent:

- swipe left or right on mobile;
- Left/Right Arrow or A/D on desktop.

Movement and route systems consume the lane-change intent without reading keys or
touch gestures directly. This keeps keyboard, touch, remote, replay, and automated
controllers interchangeable while preserving deterministic lane transitions.

### Actor, not player

`Actor` is the shared gameplay concept. It represents an entity capable of receiving actions and participating in movement or interaction.

Control is separate:

```text
Local input ──────┐
Remote input ─────┼──> ActorIntent ──> capabilities ──> motor/physics ──> body state
NPC decision ─────┘
Replay input ─────┘
```

This allows the same locomotion rules to be used by:

- the local player;
- a multiplayer remote player;
- an NPC where appropriate;
- a replay or automated test agent.

Not every NPC must use the full player motor. Sharing is a capability, not a requirement.

### Do not create one giant player system

Use composable plugins and components:

```text
aether_actor            Actor identity, body references, capability contracts
aether_control          Intents from local, remote, AI, or replay controllers
aether_locomotion       Walk, run, acceleration, jump, grounded movement
aether_rope             Rope simulation, tension, attachments, breaking
aether_grapple          Targeting, launch, attach, reel, release
aether_glide            Deploy, aerodynamic forces, stall, retract, land
aether_interaction      Generic focus and interaction requests
aether_building         Build requests and validation; not required by this game
aether_destruction      Damage/destroy requests; not required by this game
aether_actor_physics    Sole bridge between capabilities and physical body motion
```

Capabilities can be attached per actor:

```rust
commands.spawn((
    Actor,
    LocomotionCapability::default(),
    GrappleCapability::default(),
    GlideCapability::default(),
));
```

### Intents rather than direct input queries

Gameplay systems must not directly read `KeyCode` or gamepad state. A local controller translates device input into semantic intent:

```rust
struct ActorIntent {
    movement: Vec2,
    look: Vec2,
    run: bool,
    jump_pressed: bool,
    primary_pressed: bool,
    primary_released: bool,
    glide_pressed: bool,
    interact_pressed: bool,
}
```

Remote networking, NPC logic, replays, and automated tests can produce the same intents.

### One authority owns motion

Locomotion, rope, glide, wind, and future mechanics must not independently mutate `Transform`.

They produce neutral effects:

- desired movement velocity;
- impulses;
- external forces at world points;
- constraints;
- requested movement-mode transitions;
- interaction or build commands.

The actor motor/physics bridge applies them and publishes resulting body state. This is the integration seam that keeps custom mechanics modular.

### Movement modes are coordinated, not exclusive implementations

Walking, falling, rope tension, and gliding can overlap. Avoid a rigid state machine in which only one entire system runs. Use a small coordination state plus independent effects:

```text
support: grounded | airborne
posture: normal | sliding | disabled
glider: stowed | deploying | deployed
rope: detached | launching | attached | releasing
```

Rules decide which inputs and forces are available in each combination. For example, an actor can be airborne, attached to a rope, and have a deployed glider simultaneously if the design permits it.

### Multiplayer boundary

The first release is single-player, but the architecture preserves:

- serializable semantic intents;
- authoritative body snapshots;
- stable actor IDs;
- separation between presentation and simulation;
- fixed-step movement and constraint updates;
- no local-input reads inside movement simulation.

Prediction, reconciliation, transport, replication, and multiplayer product behavior are out of scope for this initiative.

## Dependency definition

### Dependency direction

```text
Bevy app/ECS/input/rendering
        │
        ├── aether_actor
        ├── aether_control
        ├── aether_geometry / collision contracts
        ├── aether_force / attachment contracts
        └── physics backend integration
                 │
       ┌─────────┼───────────┐
       │         │           │
 locomotion    rope        glide
       │         │           │
       └─────────┼───────────┘
                 │
       actor physics integration
                 │
 camera ─ route/obstacles ─ scoring/UI
                 │
        rope-glide-and-run app
```

### Required dependencies

| Dependency | Provides | Must not know about |
| --- | --- | --- |
| `aether_actor` | Actor identity, capability registration, body reference, actor state contract | Local keys, runner scoring, voxels |
| `aether_control` | Semantic intents and controller-source adapters | Physics backend details |
| Physics integration | Bodies, collision queries, contacts, forces, impulses, constraints or constraint hooks | Runner rules, voxels |
| `aether_locomotion` | Ground detection policy, acceleration, walking, running, jumping, air control | Local input devices, rope implementation, route generation |
| `aether_rope` | Rope state, tension, length, attachments, solver or backend adapter | Actors, grappling controls, runner scoring |
| `aether_grapple` | Target selection, valid anchor policy, rope creation, reel/release | Route generation, voxel storage |
| `aether_glide` | Glider state, lift/drag model, deployment and landing rules | Local input devices, runner scoring, voxel storage |
| `aether_force` | Neutral force/impulse application contracts | Source feature semantics |
| `aether_attachment` | Generic attachment endpoints and metadata | Rope or voxel internals |
| Camera system | Follow/look behavior and motion comfort | Movement implementation internals |
| Runner experience | Course rules, obstacles, pacing, score, failure, restart | Physics internals |

### Optional and later dependencies

- Voxel geometry may later provide course or collision geometry through the generic collision interface.
- Wind and Mainstream fields may later apply forces to the actor, rope, and glider.
- Interaction, building, and destruction attach as independent capabilities through commands and collision-hit metadata.
- Networking may later replace or supplement control-intent and body-state sources.

The first version must not depend on the voxel engine.

## 6. Implementation plan

The plan is intentionally parallel. Rows with the same prerequisite may proceed simultaneously.

| Milestone or task | Result | Dependencies | Status | Completion check |
| --- | --- | --- | --- | --- |
| Define actor contracts | Components and intent/effect contracts shared by controllers and movement capabilities | Bevy ECS | pending | Contract tests compile with local, AI stub, and replay-stub intent producers |
| Select physics approach | Recorded native/WASM comparison and chosen integration boundary | Geometry, force, attachment contract drafts | pending | Primitive body, sweep/raycast, force, and constraint spike runs in browser |
| Locomotion lab | Grounded walk, run, acceleration, jump, fall, and landing on primitive geometry | Actor contracts, physics approach | pending | Repeatable movement course works under fixed-step simulation |
| Rope physics lab | Tunable rope attached between generic bodies with tension diagnostics | Physics approach, attachment and force contracts | pending | Stable native/WASM swing scenarios at several speeds and lengths |
| Rope control experiments | At least three documented player-control models for attach, reel, steering, and release | Rope lab, actor intents | pending | Comparative clips and playtest notes exist; one model selected |
| Glide physics lab | Deployable glider with tunable lift, drag, stall, steering, and landing | Actor contracts, physics approach, force contract | pending | Reference gaps demonstrate controllable and repeatable glide arcs |
| Glide control experiments | At least two handling models and route consequences compared | Glide lab | pending | Selected model and tuning rationale recorded |
| Camera and comfort lab | Camera supports running, falling, swinging, and gliding without obscuring route information | Actor body state | pending | Human acceptance across the reference course |
| Greybox route | Short authored route exercises run, jump, one rope decision, and one glide decision | Locomotion, rope, glide labs | pending | Start-to-finish run is possible and recorded |
| Runner loop | Start, pacing, failure, score/time, result, and restart | Greybox route | pending | Five consecutive cycles without reload or stuck state |
| Visual identity pass | Sky setting, readable anchors, glider, obstacles, and Aether Isles presentation | Greybox route, shared visual assets | pending | Three representative clips satisfy human acceptance checks |
| Browser optimization | Reference diagnostics and accepted performance on browser matrix | Integrated greybox | pending | Performance report and automated browser smoke test pass |
| Public release | Stable URL and observation plan | All release criteria | pending | URL verified externally and release record completed |

### Parallel workstreams

After actor and physics contracts are drafted, these can proceed concurrently:

- locomotion;
- rope simulation;
- glider simulation;
- camera research;
- route visual language and modular obstacle art;
- scoring/restart shell;
- browser diagnostics and test infrastructure.

Integration should happen through the contracts rather than by merging movement logic into a single controller.

## 7. Test instructions

### Prerequisites

- Stable Rust toolchain with `wasm32-unknown-unknown`
- `wasm-bindgen-cli` version matching `Cargo.lock`
- Node.js dependencies installed with `npm install`
- Local web server started after `./scripts/build-web.sh`

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
- Expected output or generated artifact: Native checks pass, browser artifacts are generated, and route-specific browser smoke tests pass.

Additional deterministic movement and physics scenario commands will be added when those crates exist.

### Human acceptance checks

- [ ] A new player can start moving without external instruction.
- [ ] Running acceleration and jumping feel responsive rather than floaty or abrupt.
- [ ] Rope tension and the effect of release timing are visually understandable.
- [ ] The player can intentionally improve a swing rather than merely endure it.
- [ ] Glider deployment, steering, loss of speed, and landing are understandable.
- [ ] Rope and glide create route choices rather than automatic success buttons.
- [ ] Camera motion remains comfortable through a complete run.
- [ ] Failure feels attributable to a player decision and restart is immediate.

### Outcome measurement

Observe at least five first-use sessions. Record time to first meaningful action, discovered controls, use of rope and glide, completion/failure points, restarts, confusion, and spontaneous comments. Store each observation as a story and link it here before judging completion.

## 8. Release, observation, and correction cycles

### Cycle 1 — Date to be selected

- **Released:** Short authored greybox movement course
- **Audience:** Internal testers followed by a small external group
- **Expected result:** Players discover the run/jump/rope/glide sequence and voluntarily restart to improve it
- **Observation period:** Five observed sessions or one week, whichever produces useful evidence first
- **Measurements:** To be recorded
- **Feedback:** To be recorded as stories
- **Unexpected effects:** To be recorded
- **Correction or decision:** Select authored expansion, endless-runner grammar, redesign, or stop
- **Next review date:** To be selected when the greybox is released

## 9. Decision log

| Date | Decision | Evidence and rationale |
| --- | --- | --- |
| 2026-08-25 | Initiative created in `evaluating` status | The experience direction is selected, but physics, control models, and course structure require experiments |
| 2026-08-25 | Use Actor as the shared concept | Player, remote player, NPC, and replay control should be sources of intent rather than separate movement implementations |
| 2026-08-25 | Split locomotion, rope, glide, and interaction into capabilities | Custom mechanics must extend actors without growing a monolithic player controller |
| 2026-08-25 | Require one motion authority | Multiple capabilities may contribute forces and requests, but they must not race to mutate transforms |
| 2026-08-25 | Start with an authored greybox | It isolates movement feel from procedural route-generation risk |
| 2026-08-25 | Build the runner foundation before rope and glide | A dedicated greybox validates continuous forward motion, three lanes, keyboard/touch lane changes, jumping, follow camera, and character animation in isolation |
| 2026-08-25 | Use KayKit Adventurers Knight for the movement prototype | The premade CC0 glTF is lightweight, browser-friendly, and includes coherent run and jump animations; source, license, and checksum are vendored with it |

## 10. Closure

- **Final status:** Open
- **Closed on:** Not closed
- **Completion results:** Not yet measured
- **Resources used:** Not yet recorded
- **Reason for completing or discarding:** Not applicable
- **What we learned:** Pending experiments and release
- **Follow-up records:** Pending
