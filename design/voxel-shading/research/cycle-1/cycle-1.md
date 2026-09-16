# Cycle 1 — Wooden-plank surface mapping

Date: 2026-09-13

## Question

Can a wooden-plank material use voxel-body-local face coordinates to read as one crafted surface across greedy quads, irregular constructions, and moving voxel bodies without hiding the voxel scale?

## Implemented slice

- `VoxelMesh` now carries `surface_coordinates` separately from quad-local `uvs`.
- Coordinates are derived from the two axes of each voxel face and remain in voxel-body-local units.
- Wooden plank scales those coordinates so one texture composition spans 4×4 voxels.
- Bevy loads the image with repeat sampling.
- Shipwright renamed `Wood` to `WoodenPlank`; “natural wood” remains available for a later organic material.
- An automated executable renders eight fixed constructions and emits individual PNGs, an HTML index, a contact sheet, and a JSON metrics manifest.

Run the renderer from the repository root:

```sh
./scripts/render-voxel-gallery.sh
```

Open `target/voxel-render-gallery/index.html` after the command completes.

## Fixtures

| Fixture | Voxels | Vertices | Indices | Purpose |
| --- | ---: | ---: | ---: | --- |
| Single voxel | 1 | 24 | 36 | Three-face orientation and small-scale readability |
| Eight-voxel row | 8 | 24 | 36 | Long-axis continuity and repetition |
| Broad wall | 40 | 24 | 36 | Macro composition across one greedy face |
| Outside corner | 40 | 40 | 60 | Face-orientation transition |
| Window and steps | 44 | 132 | 198 | Holes, end caps, and fragmented greedy quads |
| Nominal x=16 seam | 32 | 24 | 36 | Coordinate continuity across the future chunk boundary; not yet independently chunk-meshed |
| Hull-like construction | 53 | 120 | 180 | Irregular building-scale surface |
| Moved and rotated body | 18 | 24 | 36 | Mapping remains attached to local geometry under entity transform |

## First render observation

The 4×4 mapping produced continuous grain and plank seams across broad and fragmented surfaces. Because UV origin no longer restarts per greedy quad, the hole, steps, and distant coordinate fixture do not restart the texture at each generated rectangle. Entity translation and rotation operate after meshing, so surface detail remains attached to the body.

The most visible failure was unrelated to the coordinate model: the prototype source image baked two columns of brass nails into the texture. At macro scale those became repeated interior nail bands on walls and hull-like surfaces. This conflicts with the initiative’s requirement that nails be emitted only by eligible borders.

Before correction: [contact sheet](contact-sheet-before.png)

Legacy source retained for evidence: [wooden plank with baked nails](wooden_plank_with_nails_before.png)

## Correction

The base texture was first edited to remove every nail and reconstruct wood beneath them. Review then requested a more explicit manufactured-plank treatment: a narrow bevel and recessed gap on the left and right edges, plus restrained brass nails near those edges. The selected runtime asset is now `assets/textures/shipwright/wooden_plank.png`.

This is useful visual evidence for Cycle 1, but the nails still belong to texture coordinates rather than construction topology. Cycle 2 must decide how to retain the accepted look while restricting hardware and bevel behavior to eligible borders.

After correction: [contact sheet](contact-sheet-after.png)

The first image edit used the built-in image-generation tool with this prompt:

> Remove every round brass/gold nail head from the texture and reconstruct the wood underneath naturally. Preserve the horizontal plank seams, warm dark-brown palette, grain, knots, highlights, wear, square composition, and seamless tiling. Change only nails and their immediate pixels. Add no hardware, borders, text, or watermark.

The selected follow-up edit used this prompt:

> Add a clearly visible narrow bevel and dark recessed gap along both the left and right edge so adjacent repeated panels read as separate wooden planks. Add small restrained brass nail heads near both sides, aligned within each horizontal board. Preserve the existing wood, palette, plank count, horizontal seams, and tiling.

## Decision

- Keep body-local face coordinates as a separate mesh attribute source.
- Keep the 4×4-voxel wooden-plank scale for Cycle 2.
- Keep quad-local UVs for materials that deliberately repeat per voxel.
- Use the nailed and side-beveled texture as the Cycle 1 visual target; in Cycle 2, separate that look from naive texture repetition through topology-aware borders.
- Use shading research to address dark side faces rather than encoding lighting into the texture.

## Remaining risks

- Each face chooses axes mechanically; material-authored plank orientation rules are not yet data-driven.
- The current x=16 fixture uses one sparse world and one meshing pass. True continuity between independently generated neighboring chunks remains unverified.
- The gallery records geometry counts but not mesh time, draw calls, or texture memory yet.
- Border classes, nails, AO/debug views, and a generic material catalog belong to later initiative tasks.
