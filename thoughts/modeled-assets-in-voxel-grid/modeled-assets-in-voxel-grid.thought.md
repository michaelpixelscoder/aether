---
title: "Modeled Assets in the Voxel Grid"
status: exploring
created: 2026-09-13
updated: 2026-09-13
owners:
  - michael
related_stories: []
related_actors: []
tags:
  - aether-isles
  - voxels
  - modeled-assets
  - rendering
  - world-storage
---

# Modeled Assets in the Voxel Grid

## Starting point

The voxel engine currently stores one block value per occupied cell and generates cube surfaces from that map. We also want to place authored models in the same grid: a gem, propeller, chair, machine, decorative beam, or another shape that should participate in building and destruction without being rendered as a cube.

Some modeled assets occupy one cell; others occupy several. The voxel system must know which cells they reserve, while rendering must retain one reference to the placed asset and scatter its model at the correct transform.

The initial rendering idea is:

1. generate the voxel mesh, leaving holes for modeled assets;
2. gather the modeled-asset instances;
3. render each instance at its grid-derived transform.

An optional optimization would avoid rendering a model with no path to a visible surface, such as a gem completely trapped inside opaque rock.

## Problem or opportunity

A modeled asset has two identities which the current `cell -> block` map cannot represent cleanly on its own:

- it is **grid occupancy** for placement, collision, removal, saving, damage, and queries;
- it is **one renderable instance**, even when many cells belong to it.

Storing a complete asset reference in every occupied cell duplicates state and makes multi-cell removal and updates error-prone. Storing the asset only at its anchor makes other cells appear empty unless every grid operation understands footprints. Treating its cells as ordinary opaque cubes makes the mesher generate unwanted geometry.

The opportunity is to introduce a small representation that separates occupancy from visual shape while keeping them linked by stable IDs.

## Hypothesis

Keep placed assets in a canonical instance store and maintain a reverse occupancy index from every reserved grid cell to the instance ID. Give every block or placed asset an explicit render shape (`Cube`, `Model`, or later `None`) rather than inferring rendering from occupancy.

This should let:

- grid queries see every cell occupied by an asset;
- a multi-cell asset exist only once as mutable state;
- the cube mesher omit model-backed cells;
- the renderer emit exactly one model instance at the anchor;
- placement and removal update both stores atomically;
- visibility be added without changing saved world identity.

## Exploration

### Proposed concepts

```text
AssetDefinition (catalog, immutable)
  stable asset ID
  model path / scene handle
  footprint cells in canonical orientation
  anchor and visual transform
  collision and rendering metadata

PlacedAsset (world data, canonical instance)
  instance ID
  asset definition ID
  anchor cell
  grid orientation
  instance state

CellOccupant (reverse spatial index)
  Voxel(block ID + state)
  ModeledAsset(instance ID)
```

The world would conceptually own:

```rust
struct VoxelWorld {
    cells: HashMap<IVec3, CellOccupant>,
    assets: HashMap<AssetInstanceId, PlacedAsset>,
    revision: u64,
}
```

`cells` answers spatial questions in constant expected time. `assets` is the authoritative source for modeled-asset state and iteration. A `ModeledAsset(instance_id)` cell is only a back-reference; it must not duplicate the definition, transform, health, inventory, or other mutable state.

An alternative is to keep cube voxels and asset occupancy in two maps. That may make the first adapter easier, but it creates ambiguous overlap rules and forces most callers to query both. A unified occupancy index gives placement one authoritative answer: a cell is free or it is not.

### Asset definitions and footprints

Each modeled asset definition declares a footprint as integer offsets from an anchor. A one-cell gem has `{ (0, 0, 0) }`; a two-cell bench might have `{ (0, 0, 0), (1, 0, 0) }`. Placement rotates these offsets by a discrete grid orientation and translates them by the anchor.

The footprint is simulation data, not a runtime mesh-bounds calculation. Deriving occupancy from a GLB bounding box would make placement sensitive to art edits, unused mesh padding, and floating-point rounding. A preview/debug mode should draw the declared footprint around the model so artists can validate it.

The definition also needs an authored model transform relative to the anchor. Grid orientation controls the footprint and instance transform together. Arbitrary visual rotation could be supported later, but placement orientation should initially be restricted to rotations that map integer cells onto integer cells.

### Placement and removal invariants

Placement should be a single world operation:

1. resolve the asset definition and oriented footprint;
2. reject the operation if any target cell is occupied or outside the body's allowed bounds;
3. allocate a stable instance ID;
4. insert one `PlacedAsset`;
5. write its instance ID into every footprint cell;
6. increment the affected body/chunk revisions.

Removal starts from any occupied footprint cell, resolves the instance ID, removes all its indexed cells, and then removes the canonical instance. Loading should rebuild or validate the reverse index from canonical instances rather than trusting inconsistent duplicated save data.

Useful invariants for tests are:

- every modeled-asset cell references an existing instance;
- every placed instance owns exactly its transformed footprint cells;
- no cell belongs to two occupants;
- an instance is rendered at most once;
- mutation cannot leave half an asset placed.

### Meshing semantics: occupancy is not occlusion

The cube mesher needs more information than `is_opaque(block)`. At minimum, a cell query should distinguish:

- **occupied:** may another block or asset be placed here?
- **emits cube faces:** should voxel geometry be generated for this cell?
- **occludes a neighboring voxel face:** may the neighbor's face be culled?

A modeled asset normally reserves its cells but does not emit cube faces. Whether it occludes adjacent cube faces is a separate choice.

For the first version, modeled-asset cells should be treated as empty by surface extraction: they emit no cube and neighboring cube faces remain present. This really does leave a hole for the authored model and avoids visible gaps when a model does not seal the whole cell boundary. It may generate hidden faces behind the model, but that is a safe correctness-first cost.

Later, an asset definition could provide conservative per-face occlusion masks for models that completely seal one or more footprint boundaries. The mesher could then cull only the voxel faces proven to be covered. Mesh bounds alone are not sufficient evidence of coverage.

### Rendering modeled assets

The presentation layer iterates `world.assets`, resolves each stable asset definition ID through the catalog, and derives a body-local transform from anchor, orientation, and the definition's authored offset. The voxel body's world transform is inherited by both its generated meshes and model instances, keeping movable ships coherent without rewriting grid coordinates.

For a first implementation, one Bevy scene/entity per placed asset is acceptable. If profiling later shows many repeated props are expensive, definitions can opt into GPU instancing or batching. This is a presentation optimization; it should not alter world storage.

Asset loading should be definition-driven and cached by asset definition ID. A placed instance should reference a stable namespaced ID such as `aether:resource/gem_blue`, never a filesystem path or transient Bevy handle.

### Visibility optimization

Visibility should be layered from cheap and conservative to more precise:

1. **Body/chunk frustum culling:** do not process assets in invisible spatial regions.
2. **Surface-exposure test:** render an asset if at least one footprint cell borders a cell that does not fully occlude it.
3. **Optional occlusion culling:** let the renderer reject assets hidden from the camera by other geometry.

The surface-exposure test solves the trapped-gem case without camera-dependent world logic. Cache an `externally_exposed` bit per instance and recompute it only when cells around its footprint change.

There are two possible meanings of "trapped inside rock":

- **local exposure:** no footprint cell has a non-occluding neighbor;
- **connected to outside:** empty space touches the asset but is itself a sealed cave.

Local exposure is cheap and conservative. A flood fill from the exterior can detect sealed caves, but it is more expensive and ambiguous for movable or unbounded voxel bodies. It should only be added if profiling shows that locally exposed models inside sealed volumes are a real cost. Camera occlusion culling may already cover that case.

Assets with animation, light, sound, or gameplay logic must separate **render visibility** from **simulation activity**. Hiding a mesh must not silently stop an asset's behavior.

### Chunk and revision effects

Placing or removing a model-backed asset requires remeshing chunks containing its footprint and neighboring cells whose exposed faces may change. Multi-cell footprints can cross chunk boundaries, so the mutation should collect a deduplicated set of dirty chunks after the whole operation succeeds.

The modeled instance itself does not need to be owned by exactly one render chunk, although assigning it to the anchor chunk is convenient for spatial iteration. Large assets need bounds-aware culling so an anchor outside the view does not hide geometry extending into it.

### Alternatives considered

#### Store the complete asset in every cell

Simple cell lookup, but duplicates state, risks inconsistent edits, wastes memory, and can render a multi-cell asset more than once. Reject as the canonical representation.

#### Store only an anchor marker in the voxel map

Simple serialization and iteration, but non-anchor footprint cells look empty to generic placement and collision queries. This is viable only if every relevant query performs footprint searches, which is easy to miss. Reject for runtime occupancy.

#### Represent every model footprint cell as a special block type

Fits the current map, but spreads one logical instance across blocks and still needs an external owner for shared state. It can work as an implementation detail of the reverse index, but not as the asset's canonical identity.

#### Voxelize imported models

Would unify meshing and occlusion, but loses authored geometry and is a different visual goal. It may be useful for collision or destruction proxies later, not for rendering the original modeled asset.

### Recommended first slice

Implement one static, one-cell gem and one static, rotated multi-cell asset behind a small API rather than changing game-specific placement directly:

- introduce stable asset definition and instance IDs;
- add canonical instances plus the reverse cell-occupancy index;
- add atomic `can_place_asset`, `place_asset`, and `remove_at` operations;
- adapt the mesher to query cube emission and neighbor occlusion separately;
- scatter one model entity per canonical instance;
- dirty all footprint-adjacent chunks on mutation;
- add debug footprint rendering;
- add cached local surface-exposure visibility only after the basic path is correct.

Do not begin with instancing, exterior flood fills, per-face model occlusion masks, arbitrary rotations, animation, or partial destruction.

## Assumptions

- Modeled assets occupy whole grid cells even when their visible mesh does not fill those cells.
- A multi-cell modeled asset is placed and removed atomically in the first version.
- Discrete grid rotations are sufficient initially.
- Rendering neighboring voxel faces behind a modeled asset is visually correct enough for the first slice.
- Stable asset definition IDs can be resolved through a catalog at load and render time.
- Most hidden modeled assets can be rejected by a local exposure test or ordinary renderer culling without exterior flood-fill analysis.

## Evidence

### Supporting

- The current `VoxelWorld<T>` already centralizes sparse spatial storage and revisions, so an occupancy abstraction can evolve at that boundary.
- The current greedy mesher already queries neighbor opacity to decide whether to emit faces; splitting occupancy, cube emission, and face occlusion is a local conceptual extension.
- The voxel-shading architecture already distinguishes block definitions, modeled assets, stable IDs, and presentation concerns.

### Contradicting

- No representative authored model or multi-chunk asset has yet validated footprint authoring, anchor conventions, or the proposed transform math.
- No profile demonstrates that trapped models are currently expensive enough to justify exposure caching or stronger visibility analysis.
- The current world is generic over a single block value, so a unified heterogeneous occupancy map may add API complexity before other simulation requirements are known.

## Open questions

- Can a modeled asset coexist with a voxel in the same cell, for example a wall-mounted torch or ore embedded in rock? If yes, occupancy needs layers or attachment slots rather than one exclusive occupant.
- Should neighboring cube faces always render beside a modeled asset, or do specific assets need authored per-face occlusion masks?
- Does collision use the grid footprint, an authored collider, or both for different systems?
- What is the ownership model when a multi-cell asset crosses chunks or when chunks stream independently?
- Can an asset be partially damaged, or is the full footprint atomic until a later destruction system replaces it?
- Which instance state must be saved: orientation, variant, health, inventory, animation phase, procedural seed?
- Are asset definitions global, game-specific, or namespaced catalogs composed by each game?
- Should attachments such as lamps and decals reserve a cell, occupy a face socket, or form a separate overlay system?
- Is local exposure sufficient for hidden-model optimization, or do real worlds contain enough sealed open cavities to justify exterior connectivity?

## Next experiment

Build a storage-and-meshing test without committing to the full rendering pipeline:

1. define a two-cell asset footprint with four grid orientations;
2. place it beside and inside a small opaque voxel shell;
3. assert reverse-index invariants, placement rejection, removal from either occupied cell, and dirty-chunk selection;
4. mesh the shell and verify that model cells emit no cube geometry while adjacent voxel faces remain present;
5. compute local exposure and verify that the enclosed instance is hidden and the surface instance is visible;
6. render both definitions in the existing voxel gallery to validate anchor and rotation conventions visually.

The experiment should answer the largest unresolved architectural question first: whether one exclusive occupancy layer is sufficient. Include at least one wall-mounted or embedded candidate in the fixture; if it must coexist with a structural voxel, introduce explicit attachment layers before freezing the storage API.

## Decision log

| Date | Decision | Reason |
| --- | --- | --- |
| 2026-09-13 | Create the exploration | Capture how authored models could participate in voxel-grid storage and rendering |
| 2026-09-13 | Prefer canonical asset instances plus a reverse occupancy index | Multi-cell assets need one mutable identity and constant-time cell queries |
| 2026-09-13 | Separate occupancy, cube emission, and face occlusion | A reserved grid cell does not necessarily produce or fully cover cube geometry |
| 2026-09-13 | Start with neighboring voxel faces present around models | Conservative geometry prevents holes caused by imperfectly sealing authored meshes |
| 2026-09-13 | Defer visibility until the correct placement/render path works | The optimization has no measured need yet and must not complicate canonical world data |
