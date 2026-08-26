# Aether Shipwright — concept design

## Recommendation

Build **Aether Shipwright**, a socketed modular builder presented as a shipwright's diorama in the open sky.

The tool's promise is simple:

> Pick a keel, snap on character, dress the silhouette, then capture your ship above the clouds.

The recommended first release is not a voxel editor. It uses authored modules with generous, visible sockets, optional symmetry, and a small palette of material variants. This is the fastest route to attractive first creations while preserving a neutral `ShipDesign` model and a future voxel adapter.

![Recommended builder interface](aether-shipwright-builder-concept.png)

![Target share image](cloudrunner-share-target.png)

These are directional images, not literal implementation specifications. The interaction rules and scope below are authoritative where the images differ.

## Why this fits the existing direction

The research consistently establishes:

- chunky voxel silhouettes softened by cloth, rope, vegetation, and clouds;
- dark timber and stone grounded by brass, with violet crystal as the magical accent;
- ships that read as compact floating homes rather than sleek aircraft;
- navy-and-gold framing, serif display type, and restrained diamond/compass motifs;
- bright, optimistic skies for the approachable fantasy and violet storms for dramatic contrast;
- aether currents as luminous blue-violet ribbons that connect the world;
- broad sails, balloons, rigging, lanterns, cabins, propellers, and exposed crystal engines as the most legible ship vocabulary.

The run concepts add a useful product lesson: the world is visually dense, but interactions remain obvious because collectible paths, action icons, and the player silhouette use strong color and scale separation. The builder should do the same with violet ghost parts, gold compatible sockets, and a calm navy UI shell.

## Three concept directions

### A. Shipwright's diorama — recommended

A cinematic, socketed modular builder. The ship floats large in the center; the parts tray is on the left, appearance and symmetry live on the right, and only relevant actions appear near the bottom.

- **Feeling:** toy-like, tactile, generous.
- **Construction:** authored hull, lift, drive, rigging, and detail modules.
- **Strength:** fastest satisfying first ship and strongest screenshot output.
- **Risk:** designs can look prefab if hulls are too complete.
- **Mitigation:** separate keel, bow, stern, deck, and cabin choices; allow lateral and vertical sockets; provide asymmetric detail sockets and material variants.

### B. Aether drafting table

A more precise orthographic workshop with a faint voxel grid, layer isolation, section views, and transform gizmos.

- **Feeling:** capable, deliberate, technical.
- **Construction:** hybrid modules plus small block clusters.
- **Strength:** best bridge to future voxel construction.
- **Risk:** slower first success and weaker fantasy presentation.
- **Use:** a later advanced mode, not the public landing experience.

### C. Cloud dock assembly

The ship sits beside a floating dock; users walk around it and install parts in-world.

- **Feeling:** immersive, characterful, game-like.
- **Construction:** socketed modules selected from physical racks.
- **Strength:** communicates scale and world fantasy.
- **Risk:** movement and camera navigation compete with building; implementation scope expands.
- **Use:** future game integration or optional presentation mode.

## Experience card

**Audience:** players who enjoy cozy construction, fantasy vehicles, voxel art, and sharing creations.

**First-minute outcome:** a visitor changes the starter keel, attaches lift and drive, recolors one material, and sees a recognizable ship without reading instructions.

**Core loop:** choose part → preview at valid socket → rotate/variant → place → orbit and admire → capture/share.

**Share moment:** `Capture` hides editor chrome, eases the camera to a three-quarter view, lets the user choose Dawn, Day, or Storm, and exports a clean 16:9 image. `Share design` exports or encodes the exact serialized design separately.

**Emotional arc:** empty possibility → quick competence → authorship → pride.

## Screen anatomy

| Region | Purpose | Rule |
| --- | --- | --- |
| Top bar | Name, undo/redo, save, capture | Always visible; `Capture` is the strongest action |
| Left tray | Hull, Lift, Drive, Rigging, Detail | Thumbnail-first; one open category at a time below 900 px height |
| Viewport | Build, select, orbit, admire | Ship owns at least 60% of the screen width at desktop sizes |
| Right inspector | Color/material variant and symmetry | Hidden until an applicable part or build mode is active |
| Context strip | Rotate, duplicate, delete | Appears only on selection; never covers the ship |
| Status cue | Placement validity and one-line guidance | Near the ghost part, not in a detached notification stack |

### Visual hierarchy

1. Ship silhouette.
2. Violet placement ghost or selected outline.
3. Gold compatible sockets.
4. Active catalog item and contextual actions.
5. Persistent document actions.

Panels use near-black navy with 92–96% opacity in editing mode. Brass is structure, not decoration: one-pixel rules, corners, active borders, and icons. Violet is reserved for magic, selection, and the primary capture action. Avoid ornamental frames inside the viewport.

## First-use flow

1. Open on a tiny complete starter ship named **Cloudrunner**, with the camera already orbiting subtly.
2. A single prompt says, **“Choose a bow.”** Three hull thumbnails pulse once.
3. Hovering a bow reveals compatible sockets on the ship. Clicking a thumbnail creates a ghost on the nearest socket.
4. Mouse movement cycles nearby sockets; wheel or `R` rotates through allowed orientations. Click confirms; `Esc` cancels.
5. The next prompt says, **“Make it yours.”** The selected bow's three material swatches are exposed.
6. Guidance disappears permanently after the first recolor. Undo remains visibly available.
7. When Hull, Lift, and Drive are present, `Capture` gains a quiet violet glow. It does not imply physical validation.

The starter ship prevents blank-canvas paralysis. `New design` offers **Starter ship** and **Empty keel**; the starter is the default for first-time visitors.

## Interaction contract

### Camera

- Primary drag orbits; secondary drag pans; wheel zooms.
- Double-click a part frames it; `F` frames the whole ship.
- Camera inertia is subtle and stops immediately when placement begins.
- While placing, orbit remains available from empty viewport space.
- Minimum zoom always keeps enough context to understand the attachment.

### Placement

- Selecting a catalog item enters placement immediately.
- Valid sockets appear as small gold diamonds only while relevant.
- The nearest candidate receives a violet ghost; other candidates remain gold.
- Green/blue are not used for validity. A valid ghost is violet; invalid is dim red with a short reason such as “Needs deck socket.”
- Clicking empty space does not drop arbitrary parts. Free movement is limited to parts explicitly marked `surface_placeable`.
- After placement, the new part stays selected and an unobtrusive `Place another` affordance appears.

### Selection and correction

- Click selects the smallest visible part under the pointer; repeated click cycles overlapping parts.
- Selected parts receive a thin violet outline plus their attachment socket, not a full bounding box.
- `R` rotates through authored valid steps; `Shift+R` reverses.
- `D` duplicates to the nearest compatible empty socket.
- `Delete` removes; if children are attached, choose **Remove branch** or **Keep children where possible**.
- `Ctrl/Cmd+Z` and `Ctrl/Cmd+Shift+Z` undo/redo every design mutation, including color.
- `Esc` cancels placement first, then clears selection.

### Symmetry

Symmetry is opt-in and only offered when the selected part has a valid mirrored socket. It is a placement aid, not a permanent relationship: after placement, mirrored parts are ordinary independent instances. First release needs only longitudinal mirror symmetry.

### Input and accessibility

- Every pointer action has a keyboard path.
- Visible focus rings use a pale-gold outer ring with a dark gap.
- Targets are at least 44 CSS pixels.
- Do not communicate compatibility by color alone: sockets differ by icon and ghost state.
- Tooltips appear on focus and hover, never contain required instructions, and name the shortcut.
- Reduced-motion mode disables idle orbit, cloud parallax, and capture-camera easing.

## Initial parts kit

Target **28 authored parts** plus material variants. This is deliberately smaller than the concept image suggests.

| Family | Initial parts | Expressive job |
| --- | --- | --- |
| Hull (7) | short keel, long keel, round bow, pointed bow, square stern, raised deck, cabin | Defines proportion and home/merchant/scout character |
| Lift (5) | single balloon, twin balloon yoke, broad sail, lateen sail, crystal lift ring | Changes the top silhouette dramatically |
| Drive (4) | small propeller, twin propeller yoke, aether engine, fin/rudder | Gives a readable rear and magic/nautical bias |
| Rigging (5) | short mast, tall mast, bowsprit, rope stay set, side outrigger | Connects major forms and varies span/height |
| Detail (7) | rail, lantern, banner, wheel, vent, cargo bundle, figurehead | Supports asymmetry, story, and finishing |

### Socket language

Use semantic sockets rather than a universal grid:

- `keel_front`, `keel_back`
- `deck_center`, `deck_edge_left`, `deck_edge_right`
- `mast_step`
- `lift_mount`
- `drive_mount`
- `rigging_anchor`
- `detail_surface`

Sockets have an icon, allowed categories, authored rotations, clearance bounds, and optional mirror partner. The UI never exposes socket names to ordinary users.

### Variant strategy

Each part may expose no more than three channels:

- **Primary:** timber/canvas body.
- **Secondary:** painted cloth or trim.
- **Accent:** metal or aether glow.

The first palette uses cedar, dark oak, parchment, navy, teal, rust, aged brass, iron, violet, and cyan. Keep violet crystal emissive across palettes so the Aether Isles identity survives recoloring.

## Presentation and capture

Editing uses a bright neutral Day sky for reliable contrast. Capture mode adds:

- **Dawn:** warm rim light and pale blue current;
- **Day:** crisp material readability and white clouds;
- **Storm:** deep navy clouds and stronger violet glow.

All presets share the same ship transform and camera-safe bounds. A clean capture contains the ship, clouds, distant islands, and aether current only—no title treatment, currency, HUD, or watermark. Ship name can be optionally included later as a separate, deterministic layout layer.

## Scope decision

Choose Option A from the initiative: **socketed modular parts**. Keep free voxel construction as a later experiment behind the same serialized design boundary.

This decision is justified if a primitive interaction prototype and greybox kit meet both gates:

- median first recognizable ship under 4 minutes across five new users;
- ten internal ships produce at least eight clearly distinct silhouettes when shown as black cutouts.

If either fails, revise socket topology and kit composition before adding parts. Do not add a voxel brush to rescue a weak modular kit.

## Prototype and validation plan

### Prototype 1 — greybox placement

Use 12 primitives: two keels, two bows, one stern, two lift pieces, two drives, mast, cabin, lantern. Test hover discovery, socket cycling, rotation, cancellation, selection, deletion, and undo.

Observe without coaching:

- time to first placed part;
- invalid clicks before first placement;
- whether users discover orbit without losing placement state;
- whether they can undo a mistaken attachment;
- what they believe `Capture` will do.

### Prototype 2 — creative range

Build ten ships with the 28-part kit under a 12-minute limit. Compare silhouette, lift method, hull proportion, color blocking, and asymmetry. Reject parts that only add surface noise; add only parts that open a new silhouette family.

### Prototype 3 — share loop

Run five consecutive create → save → reload → capture sessions in the browser. Compare design transforms and material variants before and after reload. Confirm capture framing for the smallest and largest fixtures.

## Out of scope for the first release

- voxel-by-voxel editing;
- aerodynamic, buoyancy, mass, or structural validation;
- damage, wetness, fire, rope simulation, or moving fabric;
- walking on the ship, piloting, combat, or crew;
- accounts, currencies, unlocks, hosted galleries, and moderation;
- freeform decals or text on sails;
- permanent symmetry constraints;
- mobile-first editing.

## Design handoff checklist

- [ ] Greybox the 12-part placement prototype before producing final art.
- [ ] Define socket metadata and overlap rules in the catalog contract.
- [ ] Treat UI thumbnails and world meshes as two views of one part definition.
- [ ] Store variants as stable IDs, never raw material handles.
- [ ] Preserve every mutation as an undoable command.
- [ ] Validate ten black silhouettes before expanding the catalog.
- [ ] Test the three capture presets on min/max ship fixtures.
- [ ] Keep Capture export and design export as separate actions and artifacts.

