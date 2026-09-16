# Image-generation execution prompt — v2

Use case: precise-object-edit

Asset type: seamless square base-color texture for a voxel-game PBR material

Primary request: refine the existing wooden-plank texture so it contains exactly
five complete horizontal wooden planks and no partial sixth plank at the bottom.
Preserve the established dark warm walnut, hand-painted realistic grain, beveled
left and right ends, and exactly two restrained round aged-brass nails per plank.

Composition/framing: perfectly orthographic and axis-aligned. The canvas begins
at the outer/top bevel of plank 1 and ends at the outer/bottom bevel of plank 5.
Five equal-height plank bands fill the canvas exactly. Internal seams occur only
between those five bands. No new plank begins below the fifth plank. Left and
right outer bevels terminate exactly at the image bounds.

Lighting: flat neutral base-color authoring light; no cast shadows or directional
scene lighting.

Constraints: preserve continuous horizontal grain and the current visual style;
exactly ten nails total in two aligned columns; complete border on all four sides;
no content outside the outer bevel; no cropped or partial plank; no perspective;
no text; no watermark. Make the border clean and geometrically consistent enough
for deterministic pixel framing afterward.
