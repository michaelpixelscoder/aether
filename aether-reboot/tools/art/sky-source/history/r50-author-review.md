# R50 — isolated original low-sun source

This experiment replaces the Earth/Hosek composite with a planetless World:
one real uniform-radiance stellar emitter at -2.921837deg and a smooth broad
blue boundary radiance. The same blue field illuminates the VDBs and appears
in the background. The existing finite air computes localized scattering;
there is no synthetic halo. Geometry, camera pose and all86VDBs are unchanged.

`build_low_sun.py --prepare` opens the r49 source and writes only this stage.
The old SUN remains in the file as a hidden historical object; it contributes
no light. No old Nishita/Hosek node is active in the new World. The active disk
has .75deg diameter, RGB(1,.62,.32), red-channel normal irradiance22 and uniform
red radiance `22/(pi*sin(.375deg)^2)`.

The22 calibration is grounded in the previous rendered source: numerical
integration of the old solar cone gives red irradiance14.094 through the
panorama's existing air. Multiplying by the old World lighting/camera ratio
.09/.10 gives12.685, then adding the old SUN9.3 gives21.985. This is an
approximate reference, not exact energy equality at every cloud location.
The new source replaces those two contributions rather than adding a third.
Exposure stays-.90EV.

The broad blue boundary is explicit artistic radiance, not a strict physical
planetary atmosphere. Its linear RGB is(.04,.09,.24) at the lower pole,
(.055,.13,.30) at the upper pole, and(.24,.50,.94) around horizontal rays.
The horizon weight is exp(-4*z*z); there is no discontinuity at z=0 or seam.
Camera and illumination use identical blue radiance, ratio1.

The refreshed CPU cloud audit uses current cloud-banks.json, shader and macro
volumes, all hashed in occlusion-upper-bound.json. At the target(295,75) its
sampled conservative density bound gives transmission>=.999963 at gallery
time120s. This audits runtime clouds, not baked VDBs or opaque island depth.

One original1672x94164-sample hero is authorized. The script refuses to
overwrite its output. It also saves the same Render Result as native linear
HDR for numerical inspection; there is no second render, resize or retouch.
The hero uses AgX Medium High Contrast; game ACES must be reviewed separately.
Manifest output_needs_render remains true until an original panorama exists.

The future IBL switch is the input1 of the node named
`IBL omit direct camera disk | 0 hero, 1 future IBL`. With value1 only direct
camera visibility of the stellar surface is removed. Its light still reaches
VDBs and the finite air via identical volume/shadow paths. This stage does not
render or publish that IBL before hero review. A1024x512 original linear HDR
would be about6MiB decoded RGB32F, or4MiB if packaged as RGBA16F, before any
offline cubemap conversion. Its authoring exposure must remainRaw/0EV, with
the same runtime radiance calibration used for the visible panorama.

No canonical asset, Rust, shader, documentation or source was changed.
The manifest retains all86source hashes, five frozen canonical hashes, actual
scene descriptors and the r49 parent source hash. Render completion and
numerical results will be recorded in that manifest.

## Completed hero and honest review

The single hero completed in67.746s on2026-10-03. The source is visible where
the unchanged camera predicts: bright-pixel centroid(294.55,74.40), bounding
box(284,64)–(305,84). Native linear samples are finite and nonnegative,
range[.099609375,179200]. This does not prescribe final KTX encoding; a future
original panorama must choose its encoding from its own full-sphere maximum.

Author review: the true source and creamy lit edges are present; the accepted
connected cloud shapes and blue interiors survive. The upper-left sky is
paler than r49 and the broad golden opening remains weak. This is a useful
structural lighting result, not a claim of reference fidelity or game approval.
The hero has no game islands or runtime cloud banks. No additional render is
authorized or performed before the parent's review.

GPU process45660 exited and the GPU window was released. All97 source/output
hash checks passed, including the five frozen canonical files. Source-audit
and the manifest preserve the exact hashes and original native render paths.
