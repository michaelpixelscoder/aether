# R53 — finite-air forward-scattering comparison

This isolated original hero starts from the untouched R52 corona candidate.
The parent authorized exactly one material input change, from .72 to .90 on
`R43 | finite aerial scattering` / `Volume Scatter.001` / `Anisotropy`.
The preparation script compares complete descriptors and asserts that this
is the sole difference. All 98 VDB fields, their materials and placement,
both air densities and colors, the actual .75-degree stellar source at
(-.853562646464, -.050973576023, -.518490600791) in game coordinates,
irradiance 22, camera and -0.90 EV remain fixed.

The real source and its scattered light share the same scene. There is no
camera-specific halo, painted disk, altered image or added emitter. This
remains the artist-directed planetless atmospheric model documented in R50.

The diagnostic preceding this experiment is retained intact in
`../sky-r52-corona-candidate/column-audit.json`, `analyze_columns.py` and README.
It found no VDB density at the source or along the measured vertical clear
cone. New sculpted margins instead have g=.70, not the original cumulus g=.35.
The finite air therefore supplies the scattering inside this opening.

Normalized Henyey–Greenstein predicts a change in phase half-maximum radius
from 14.53 degrees at g=.72 to 4.63 degrees at .90. The phase density increases
8.66 times on axis and 5.94 times at 3.46 degrees, while falling to 47.2% at
27.6 degrees. The reference's roughly 180-pixel opening spans approximately
plus/minus 3.5–3.8 degrees. These are analytic single-scattering phase values,
not a prediction of rendered pixel values: extinction, integration through
the scene, diffuse illumination, multiple scattering and the display
transform still matter. Source power and total normalized scattered power
are not increased.

The blue component remains g=.15/density 2e-6/color (.34,.55,1). The changed
neutral component retains density 3e-7 and color (.98,.985,1). Blue particle
color multiplied by the warm direct source approximates (.34,.341,.32),
which explains why simply increasing that scattering would add a nearly
gray direct-key veil. The neutral component preserves the warm source ratio.

`build_forward_air.py --prepare` saves an isolated source with relative VDB
references. `--hero` allows one original 1672×941, 64-sample OPTIX render,
RGB-only CPU denoising and the same AgX Medium High Contrast display transform
as R52. There is no panorama export mode. The base R52 and canonical/r50 files
are hash-frozen and verified after rendering. `output_needs_render` remains
true until a separately authorized panorama exists.

The source, generator, input volumes, before/after descriptors, render log
and image hashes are recorded in `manifest.json` and `audit.json`. R52's
original hero is preserved for the comparative visual review.

## Result and comparative self-review

The one authorized hero completed without an error in 69.20 seconds. Process
14732 exited and the GPU was returned to the parent before this CPU review.
The original image is `forward-air-hero.png`, 1672 × 941, SHA-256
`dce9d90d29d3fc4791ce49ac85644df9f89aaccb65b6ec8687c14ebb874a8217`.
Source SHA-256 is
`2aaaeda063c6012ae9f8068e5986b9412060deba78258b4658807219d00a3c66`.

The hypothesized directional effect is visible: the real source is now
surrounded by an illuminated opening rather than a nearly uniform slate
background. Its light is concentrated in the upper left and the distant
clear areas become slightly deeper blue. The same cloud geometry and small
articulations remain recognizable; the phase change does not create new
silhouettes or a larger solar disk.

The visual target is still only partially met. The new opening reads mostly
white/cream with a broad, soft falloff, not the reference's clearly golden
opening and sharply articulated bright margins. It also veils some of the
nearby sculpted fragments. This is evidence that the missing scattering
mechanism has been identified, not evidence that the final color and breadth
have landed. It is not yet a successful whole-scene judge result. AgX here
and the engine's own display transform are distinct, so the source hero
alone cannot establish the final engine hue.

`display-comparison.json` records read-only 9×9 mean patches from the existing
PNG files, converted by System.Drawing to 8-bit display RGB. No image was
written or modified by this comparison. At clear ray (265,75), R52's
(137.5,140.2,150.5) becomes R53's (219.5,216.0,212.5). At (385,75), it changes
from (135.0,138.6,150.4) to (205.6,202.2,199.4). This confirms a large real
brightness increase but only modest warm display chroma. The far clear
patch (1550,700) shifts only from (44.9,71.7,107.6) to (43.4,71.0,107.0),
consistent with the predicted reduction in wide-angle veil. These display
values must not be interpreted as linear radiance ratios.

`audit.mjs` passes 126 SHA records covering 125 unique files, including all
12 frozen canonical/r50/R52 files. The complete descriptor differs at exactly
the authorized anisotropy input, and all 98 VDB fields remain identical.
No panorama, IBL, format change or canonical promotion has been performed.
R52 and the phase prediction remain intact for the parent's next decision.
