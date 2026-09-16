# Wooden plank — albedo brief

## Positive prompt

Orthographic square game-material texture, five broad horizontal dark walnut
wooden planks, warm hand-painted realism, continuous longitudinal grain, subtle
age and edge wear, narrow recessed seams between boards, beveled left and right
board ends, exactly two restrained round forged-brass nails per plank near the
left and right borders, even neutral illumination, no cast shadows, physically
plausible surface, sharp clean detail, seamless periodic top/bottom and
left/right boundaries.

## Negative prompt

Perspective, furniture, wall scene, diagonal boards, vertical boards, text,
logos, extra hardware, screws, nail drift, missing nails, broken geometry,
dramatic lighting, large highlights, baked shadows, dirt blobs, moss, water,
plastic, photogrammetry edge, incomplete plank at a texture boundary.

## Structural contract

- Horizontal seams repeat at one-fifth texture height.
- Left/right bevels and two nail columns repeat at identical normalized offsets.
- Nails are metal; wood and gaps are dielectric.
- Wear may brighten bevel crests and dull nail crowns, but must not move geometry.
- The height map is the canonical source for normal and AO.
- OpenGL tangent-space normals are used by Bevy.
