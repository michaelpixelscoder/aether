---
title: "Fabric and Rope System"
status: exploring
created: 2026-09-13
updated: 2026-09-13
owners:
  - michael
related_stories: []
related_actors: []
tags:
  - aether-isles
  - fabric
  - rope
  - physics
  - voxels
  - shipwright
---

# Fabric and Rope System

## Starting point

Concept sheets [`aether_isle_14.png`](../../design/initial_researches/aether_isle_14.png) through [`aether_isle_20.png`](../../design/initial_researches/aether_isle_20.png) propose one flexible construction language for sails, flags, canopies, balloons, trampolines, ropes, rigging, pulleys, damage, wetness, and fire.

The repeated idea is more important than any single pictured mechanism:

- fabric is a two-dimensional lattice of visible voxel-like tiles;
- rope is a one-dimensional chain of short visible segments;
- both attach to rigid voxel structures through explicit anchors;
- tension gives structures their shape and transmits load;
- failure removes connections and changes gameplay rather than only playing an effect.

The references call the surfaces “voxel fabric,” but they depict a stylized flexible structure rather than ordinary volume voxels. Treating it as its own simulation type should preserve that distinction.

## Problem or opportunity

Rigid grid blocks alone cannot express the large readable motion that gives an airship wind, weight, and fragility. A general cloth solver, however, would introduce behavior and visual noise that conflict with the chunky Aether Isles style.

The opportunity is a deliberately constrained system: enough physics to communicate force, tension, sag, lift, and failure, with a rendering layer that continues to look constructed from discrete pieces.

If fabric and rope share particles, constraints, attachments, force application, and breakage, sails and rigging can form one load-bearing network instead of two unrelated visual effects.

## Hypothesis

Represent rope and fabric as specializations of one position-based constraint graph:

- **nodes** hold current and previous position, inverse mass, and accumulated influences;
- **distance constraints** connect neighboring nodes and preserve rope length or fabric spacing;
- **optional diagonal constraints** resist fabric shear;
- **optional bend constraints** resist sharp folding across two rope links or adjacent fabric rows;
- **anchors** bind nodes to rigid voxel bodies or modeled attachment points;
- **material state** controls compliance, mass, damping, strength, wetness, heat, and damage;
- **broken constraints** produce tears, snapped rope, and detached regions.

Run this graph at a fixed simulation step and solve constraints iteratively. Render rope segments and fabric tiles from the solved graph, rather than moving voxel-world cells. This should be deterministic enough for construction gameplay, inexpensive enough for an initial CPU implementation, and extensible without committing to a full continuum cloth model.

## Exploration

### The common core

```text
Rigid voxel body / attachment socket
              │
           Anchor
              │
    Simulation node graph
      ├─ 1D chain → rope
      └─ 2D grid  → fabric
              │
      solved positions + tension
              │
    stylized render reconstruction
```

Rope is not a separate physics problem at the first layer. It is a graph whose nodes normally have degree two. Fabric is a graph laid out as rows and columns, with optional diagonals and bend relationships. Sharing the graph makes rope-to-fabric edge bindings, tethered corners, and force transfer natural.

The simulation graph should be body-local when attached to a moving ship. That avoids injecting the ship's large world translation into every flexible node and keeps floating-point behavior stable. External forces such as gravity and wind can be transformed into body-local space for integration. Detached islands can be promoted to world-space dynamic objects if gameplay needs them to persist.

### Fabric should remain visually voxel-like, not physically axis-aligned

Sheet 14 says cells remain square and axis-aligned while neighbors slide. Taken literally, this cannot produce all the curved sails, balloons, and canopies shown later without gaps, overlap, or stair-stepping. A rigid square also cannot follow a compound-curved balloon while remaining globally axis-aligned.

Use that statement as an art constraint instead:

- preserve a clearly discrete square-tile silhouette;
- limit stretch so tiles remain nearly square;
- allow each rendered tile to orient from the local deformed surface frame;
- retain small seams or cords between tiles so bending is legible;
- quantize or damp normals if a softer cloth look emerges.

The physical nodes may move continuously even though the render remains blocky. Simulation resolution and visible tile resolution can initially be one-to-one, but they should not be permanently coupled in the data model.

### Rope rendering

Render one short cuboid, braided module, or authored link between each pair of solved nodes. The segment aligns to the link direction and exposes tension through controlled straightening. A rope at rest may use a little authored angular irregularity, while a loaded rope becomes visually ordered.

The concept images sometimes resemble chains more than fiber rope. That can be a material/render profile on the same solver:

- rope: lighter, compliant, damped, frays before failure;
- chain: heavy, low stretch, articulated links, abrupt break;
- aether cable: emissive and perhaps capable of transmitting energy.

### Fabric topology and borders

A rectangular fabric definition contains width, height, rest spacing, material, and a set of active cells and links. Not every cell must exist, allowing triangular sails and damaged silhouettes without forcing a rectangular render.

Each visible tile can be reconstructed from four neighboring nodes, or a node-centered local frame. The four-node form makes tears explicit but can warp a tile; the node-centered form keeps tiles rigid but introduces gaps. The references favor rigid-looking pieces with visible seams, so a node-centered rigid tile is the better first visual experiment.

Borders should be explicit. A reinforced edge rope can be a real chain of constraints bound to the fabric boundary, with stronger material values and attachment sockets. This yields the heavy framed edges seen in sails, balloons, and trampolines and gives damage a readable progression: surface links fail before the reinforced border.

### Attachments and anchors

Do not address attachments by raw entity IDs or arbitrary world positions in saved data. A flexible structure should bind to stable sockets on a voxel body or placed asset. A socket can resolve at runtime to a body-local transform.

Useful anchor modes are:

- `Fixed`: node follows one socket exactly;
- `Edge`: a row of nodes maps between two sockets or along a rigid beam;
- `Tethered`: node connects to a socket through a rope constraint;
- `Sliding`: node may move along an authored rail or rope parameter, later;
- `Free`: no current attachment.

The MVP needs only `Fixed` and `Tethered`. A full sliding attachment or pulley is substantially harder than a fixed redirect point and should not be implied by the first version.

### Pulley scope

A convincing pulley preserves the total length of a rope while contact points slide around a wheel and forces transfer between both sides. Modeling this robustly across changing collision contacts is its own subsystem.

For the first construction experiment, treat a pulley as a rigid sequence of guide sockets. Rope passes through these kinematic points, changes direction, and preserves the combined rest length of the spans. This proves rig layouts and force redirection. Free sliding, wrap angle, wheel rotation, friction, winches, and reeving can follow only if they become important gameplay.

### Forces and gameplay outputs

The solver should consume simple, inspectable inputs:

- gravity;
- wind velocity or pressure;
- impulses from characters and cargo;
- anchor motion from the rigid body;
- changes in rest length from a winch or construction action.

Fabric needs a coarse aerodynamic force. For each active tile, compare its normal with relative wind and apply pressure across its area. The first model does not need fluid simulation. Its useful outputs are total force and torque on the owning rigid body, plus per-link strain for animation and failure.

The same graph can report:

- current tension at a rope link or anchor;
- sail force and approximate center of pressure;
- trampoline compression and released impulse;
- detached connected components;
- whether a functional surface has enough intact area to operate.

### Damage, water, and fire

The later sheets are a desired progression, not MVP scope. The graph nevertheless provides good extension seams:

- **damage:** reduce link strength, then remove a selected constraint;
- **tear:** allow breakage to propagate where neighboring strain is high;
- **water:** increase node mass, damping, and visual darkness; reduce lift or bounce through gameplay coefficients;
- **fire:** spread heat across graph adjacency, weaken hot links, and remove tiles after failure;
- **detachment:** find connected components no longer reachable from an anchor.

Topology changes should occur at bounded synchronization points rather than during constraint iteration. This keeps solver arrays stable and makes failures easier to replay and debug.

### Data boundary

A likely definition/runtime split is:

```rust
struct FlexibleDefinition {
    topology: FlexibleTopology, // Rope or fabric grid/mask
    material: FlexibleMaterialId,
    render_profile: FlexibleRenderProfileId,
    bindings: Vec<DefinitionBinding>,
}

struct FlexibleInstance {
    definition: FlexibleDefinitionId,
    body: Option<VoxelBodyId>,
    nodes: Vec<FlexibleNode>,
    constraints: Vec<FlexibleConstraint>,
    bindings: Vec<ResolvedBinding>,
    state: FlexibleState,
}
```

Definitions describe rest topology and authoring intent. Instances own deformation, breaks, wetness, and heat. Saving every node position may not be necessary for intact resting structures; damaged topology and dynamic state can be saved separately and the pose reconstructed, but this should wait until the runtime behavior exists.

This belongs beside the voxel feature rather than inside `VoxelWorld<T>`. It references voxel bodies and sockets, while the voxel map remains authoritative for rigid occupancy. Fabric should not reserve every swept cell it currently intersects.

### Solver direction

Position-Based Dynamics or XPBD is a good initial fit because construction games value stability and controllable constraint softness more than exact forces. XPBD's compliance is less dependent on time step and iteration count, which matters when materials such as canvas, rope, and chain need repeatable tuning.

A fixed step, a small bounded substep count, and deterministic iteration order are more important initially than parallelism. Broad collision can begin with a ground plane and a small set of rigid primitives. Per-voxel collision, self-collision, arbitrary cloth-cloth collision, and network replication would each multiply scope.

### Alternatives considered

#### Animate a skinned mesh

Cheap and art-directable, but it cannot naturally transfer tension, reconfigure between arbitrary construction points, or produce topology-driven failures. Useful for distant decorative fabric, not the construction system.

#### Make every cloth tile a rigid body with joints

Visually literal, but entity, collision, and joint counts would grow quickly. Solver jitter would also undermine sails and trampolines. Reject for the core surface simulation; detached chunks could later become rigid debris.

#### Use a conventional continuous cloth mesh

Mature and smooth, but fights the tile language and makes authorable tears and voxel attachments less direct. A continuous mesh may be a future LOD, not the authoritative structure.

#### Keep cells globally axis-aligned and slide them

Strongly preserves voxel purity, but cannot reproduce the concept art's curvature without severe overlap and gaps. Worth testing as an extreme art variant, not adopting as the simulation rule.

## Assumptions

- Flexible structures are authored objects attached to voxel bodies, not blocks stored inside the sparse voxel map.
- The important visual promise is discrete constructed tiles, not globally axis-aligned tiles.
- Small amounts of stretch are acceptable if the system communicates tension clearly.
- Initial scenes contain tens of structures or a few thousand simulation nodes, making a CPU solver plausible.
- A coarse aerodynamic model is sufficient to make sails affect airship motion.
- Self-collision and fully physical pulleys are not required for the first playable slice.
- Fixed-step simulation and stable IDs will matter if replays or multiplayer arrive later.

## Evidence

### Supporting

- Sheets 14–18 consistently depict a shared 2D fabric, 1D rope, rigid anchor, and tension vocabulary across otherwise different structures.
- Sheets 19–20 define failure primarily as connection loss, changed tension, and detached pieces, which maps directly to constraint-graph topology.
- The existing `aether_voxels` crate currently owns sparse rigid block storage and meshing but no rigid-body identity or flexible simulation, leaving a clean boundary for a separate feature rather than forcing fabric into voxel occupancy.
- Visible seams, reinforced borders, and blocky links in the references allow a low-resolution simulation to be an intentional style choice.

### Contradicting

- The concept sheets do not demonstrate a running solver, performance target, construction editor, collision behavior, or save format.
- The claimed combination of globally axis-aligned square cells and smoothly curved surfaces is geometrically inconsistent.
- Balloons require closed-surface pressure or a strong shape-restoring model; they are materially harder than open sails and should not be treated as an automatic use case.
- Trampolines need character contact and reliable impulse transfer, while sails need aerodynamic coupling. Sharing topology does not make all gameplay integrations free.
- True pulleys and winches need continuous rope-length redistribution that ordinary fixed distance constraints do not provide by themselves.

## Open questions

- Are fabric tiles physical panels with gaps, or merely a rendering of a continuous underlying sheet?
- Must fabric be editable tile-by-tile during construction, or selected from authored shapes and resized?
- Are airships simulated as rigid bodies already, and where should force/torque from sails be applied?
- What is the maximum expected node count per ship and per visible world?
- Do ropes collide with voxel geometry, wrap around it, or only interact through explicit anchors and pulley sockets?
- Can players cut any rope segment or fabric link, and must the result be deterministic across multiplayer peers?
- Should detached fabric remain simulated, convert to simplified debris, or disappear after a short lifetime?
- Does a damaged sail's effectiveness depend on intact area, orientation, graph connectivity to the mast, or all three?
- Is balloon gameplay important enough to justify volume preservation and pressure constraints in the first system?
- How visibly voxel-pure should a bent tile remain: freely oriented rigid panels, quantized orientations, or globally axis-aligned sliding blocks?

## Next experiment

Build one isolated `aether_flexible` gallery scene before integrating with Shipwright:

1. create a 12 × 8 rectangular fabric graph and a 16-segment rope with XPBD distance constraints;
2. pin the fabric's upper corners and tether its lower corners through the rope chains to four moving debug anchors;
3. render node-centered rigid square tiles and cuboid rope links, with a tension heatmap debug mode;
4. apply gravity and a constant directional wind, then move one anchor through a scripted path;
5. cut one rope link and one interior fabric constraint, detecting newly detached components;
6. compare three tile treatments: local free orientation, quantized orientation, and globally axis-aligned sliding;
7. record fixed-step stability and frame time at 100, 1,000, and 5,000 nodes on native and WebAssembly builds.

This experiment should answer the most consequential visual question before architecture hardens: whether a low-resolution constraint surface can move convincingly while still reading as voxel-built. It intentionally excludes voxel collision, self-collision, balloons, water, fire, true pulleys, and production construction tools.

Success criteria:

- the sheet responds to wind and anchor motion without explosive instability;
- rope visibly straightens under load and transfers motion to the fabric;
- tiles remain clearly discrete at gameplay camera distance;
- cutting a connection creates a predictable local or connected-component failure;
- the same material tuning behaves comparably across the chosen fixed step and substep budget;
- 1,000 nodes leave adequate frame budget for the rest of the web game.

## Decision log

| Date | Decision | Reason |
| --- | --- | --- |
| 2026-09-13 | Start the exploration from concept sheets 14–20 | They establish the intended fabric, rope, attachment, and failure vocabulary |
| 2026-09-13 | Treat fabric and rope as one constraint-graph family | It naturally supports shared anchors, tension, load transfer, and breakage |
| 2026-09-13 | Keep flexible structures outside voxel occupancy | Their nodes move continuously and should reference rigid bodies rather than masquerade as static cells |
| 2026-09-13 | Interpret “voxel” as the render language, not global axis alignment | The pictured curved structures are incompatible with literal axis-aligned rigid squares |
| 2026-09-13 | Prototype XPBD with fixed and tethered anchors first | It tests the central motion and visual hypothesis while bounding solver scope |
| 2026-09-13 | Defer balloons, self-collision, true pulleys, wetness, and fire | Each adds a distinct hard problem before the common core is validated |
