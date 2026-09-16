# Wooden plank texture prompt

Exactly five complete, equal-height horizontal dark-walnut planks in a square
orthographic source texture. Each source plank has one uninterrupted horizontal
grain field, complete left/right bevels, and two restrained aged-brass nails
near its outer edges. It will be deterministically packed into complete 1×1,
1×2, 1×3, 1×4, and 2×5 greedy-quad templates; do not generate separate tiles.
Internal horizontal seams exist only at 20%, 40%, 60%, and 80% of image height.

The top image edge is the outer top bevel of plank one. The bottom image edge is
the outer bottom bevel of plank five. Do not begin or reveal a sixth plank below
the bottom bevel. No partial planks, perspective, baked lighting, cast shadows,
text, logos, extra fasteners, or details crossing the outer frame.

The generated image is one continuous master source. `source/prepare_inputs.py`
owns the pixel-perfect 1280×1280 framing, canonical 256-pixel plank grid, and
packing of complete greedy-quad templates into the final atlas.
