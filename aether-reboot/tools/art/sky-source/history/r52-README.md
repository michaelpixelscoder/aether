# R52 — fragmented original cloud corona, isolated hero

The parent accepted r50's real low solar source and r51's runtime cloud-bounce
lighting, while requesting a sharper cloud hierarchy, saturated clear blue,
and a broken warm opening around (295,75). This experiment keeps the accepted
camera, actual .75-degree emitter, source irradiance22, RGB(1,.62,.32), −.90EV,
the finite aerial medium and all86 existing VDB masses/materials unchanged.
The r50 package and canonical source/output are frozen and checked by SHA.

Two structural changes are authored together:

1. Four asymmetric branches near the actual solar direction, plus eight small
   remote weather shreds. These are223 affine placements of the existing ten
   original3D density sculptures, fused into12 independent continuous fields.
   Depths range roughly18–43km, with severalfold size variation, gaps and thin
   fingers. No closed ring or single horizontal shelf is created. The recorded
   planning pixels are converted into fixed3D coordinates; there is no camera
   billboard, screen-space density mask or bitmap layer. A complete .72-degree
   half-angle cone stays clear of the added sculptures around the .375-degree
   solar radius.
2. Remove the explicit radiance peak formerly centred on z=0 in r50's World.
   With this downward camera that peak covered almost the entire upper frame,
   washing the clear sky pale. The unchanged blue pole radiances now directly
   form one continuous full-sphere field for visibility and all lighting paths.
   Source, exposure, finite scattering densities and old volume materials stay
   unchanged. This remains an artist-directed planetless atmosphere model.

The new optical-thin cloud margins use actual participating volumes, density
.0022 with anisotropy.70; remote torn vapour uses.0013/.68. Both retain the
existing cloud scattering/absorption colors and zero emission. Their warm
appearance must come from the unchanged real stellar emitter. No halo or
enlarged disk was added.

`build_corona.py --prepare` rebuilds the isolated source from r50 and records
every affine transform, input SHA and output VDB SHA. The 12 fields occupy
868,907 bytes; peak dense union scratch is20,642,360 bytes. Near fragments use
14m voxels and remote shreds20m. Every field is finite, bounded in[0,1] and has
an explicitly verified zero boundary. Construction took10.39s with4CPUthreads.
The first preparation exposed a relative-path error before scene validation;
its script/error are preserved in history. Corrected preparation uses absolute
paths until saving the candidate, then writes relative references and checks
that its descriptor is unchanged. No failed candidate was rendered.

The current runtime-cloud support audit is independent of the authoring render.
At gallery time120s the stellar ray has conservative sampled transmission
>=.999963. Lower/eastern branches have empty cloud support at their audit
axes. Four upper-western connection points remain indeterminate behind the
west bank; the panorama cannot replace that runtime silhouette. This is not
an opaque-island-depth test, and a small lower transmission bound does not
prove an opaque result.

Only one original1672×941/64 hero is authorized, with AgX Medium High Contrast.
No4K,8K,IBL export, image resizing, retouching or canonical promotion occurs.
The original4K texture's angular sampling remains a separate limitation for
engine sharpness; the parent owns a possible later format/resolution study.
The resulting authoring hero must be reviewed before any panorama export.

## Completed hero and honest review

The single original hero finished in 71.43 seconds without a renderer error.
`corona-hero.png` is 1672 × 941 at 64 maximum samples, with no resize or image
editing. SHA-256: `0d64cae1dd15a10b60ae36125e4cf6152dbd0db8f58902cedf03b5ee22cdfe2d`.
GPU ownership was returned to the parent immediately after process 10204 exited.

Against r50 and the complete r51 game captures, the clear blue is deeper and
the new fragments supply useful smaller articulations around the stellar
opening. No old volume boundary faces have reappeared. The original broad
cloud silhouettes remain intact. This is useful progress, not a completed
match to reference 33: the new fragments are mostly cream, some read as flat
scalloped plates, and the approximately 180-pixel golden opening is still
absent. A source-only hero also cannot prove the result behind runtime banks
and islands. It is not ready for a successful full-scene judge submission.

`node .dream-loop/sky-r52-corona-candidate/audit.mjs` passes 134 hash records
covering 121 unique files. All eight frozen canonical/r50 files, 88 original
objects including 86 volume fields, three original materials, five solar
nodes, camera and exposure remain unchanged. The only added objects are the
12 documented participating volumes. `output_needs_render` remains true:
there is no candidate panorama and no promotion.

## CPU diagnosis of the missing golden opening

`analyze_columns.py` samples the actual transformed VDB grids along 19 fixed
gallery rays. Its 17,554 nearest-voxel samples took 0.25 seconds and are
recorded in `column-audit.json`. Step length is at most 0.7 index voxel.
This is a density-column diagnostic, not a radiance or full transmittance
solution: it excludes colored extinction, shadow rays and multiple scattering.
The OpenVDB binding reported retained Python instances during interpreter
shutdown after writing the complete result; this was not a render failure.

The disk ray (295,75) contains no authored cloud density. All five measured
vertical rays through x=295 from y=15 to 165 are also empty, as are horizontal
rays at x=265 and 325. The actual near-source columns instead sit at x=205,
235 and 355, where density-coefficient × length is approximately 0.340,
0.215 and 0.223. These are dominated by the **new g=.70 material**, not by the
old g=.35 cloud body. Changing the old body phase would affect the large
clouds but cannot fill this central scattering gap.

The old body material really is g=.35/density .009; old thin vapour is
.65/.0015; the added margins are .70/.0022; remote fragments are .68/.0013.
The finite air contains a blue scatter component, color (.34,.55,1), density
2e-6, g=.15, plus a nearly neutral component, color (.98,.985,1), density
3e-7, g=.72. Both use normalized Henyey–Greenstein phase functions. Only this
finite air occupies the measured clear rays close to the source.

For a .75-degree emitter, the small-angle single-scattering approximation
gives the following phase values. These are analytic phase densities, not
rendered luminance or a promised final exposure.

| Phase g | On-axis sr^-1 | At 3.46° sr^-1 | Half-maximum angular radius |
| --- | ---: | ---: | ---: |
| .72, current neutral air | 1.746 | 1.661 | 14.53° |
| .85 | 6.543 | 5.389 | 7.15° |
| .90 | 15.120 | 9.869 | 4.63° |
| .92 | — | — | 3.66° |

The requested 180-pixel diameter corresponds to roughly ±3.5–3.8° around the
source, whereas the current neutral-air phase distributes its power across a
much wider angle. At 27.6°, g=.90 has only 47.2% of the current .72 phase;
on-axis it has 8.66 times as much. Changing this normalized angular
distribution therefore concentrates existing scattered energy near the real
source while reducing the wide veil; it does not increase source power or
add a camera-aligned halo.

There is also a specific color cancellation: in the single-scattering limit,
blue-air color (.34,.55,1) multiplied by the warm source (1,.62,.32) becomes
approximately (.34,.341,.32), nearly neutral. Its scattered key component
cannot by itself produce saturated blue or amber. The neutral particles
preserve the source's warm spectral ratio much better. Ambient blue and
multiple scattering still matter, so this is a causal diagnosis, not a full
color prediction.

Recommended next isolated experiment, pending parent authorization: retain
all R52 geometry, source and exposure, and change only the neutral finite-air
phase from .72 to .90. Keep its density and color, the blue-air component and
cloud materials unchanged. This is a targeted diagnostic justified by the
measured empty cone and incorrect angular width, not another key/exposure
increase. It should be judged for an actual warm scattered opening and clear
blue away from the source before any original panorama or matching IBL is
rendered. If it merely washes out the opening, reject it; do not raise source
intensity or add a bitmap/screen-space glow. No such material change has been
made in the present candidate.
