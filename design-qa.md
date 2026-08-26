# Design QA — Aether Shipwright voxel editor

- **Source visual truth:** `design/skyship-building-tool/aether-shipwright-builder-concept.png`
- **Implementation:** `/game/shipwright/` from the local `dist` build
- **Intended viewport:** 1280 × 720 CSS px at device scale 1
- **Source dimensions:** 1672 × 941 px
- **Implementation screenshot:** unavailable; the T3 collaborative preview initialized the 1920 × 1080 WebGL canvas and displayed the live route, but `preview_snapshot` failed repeatedly at 1280 × 720 and 960 × 600. A browser recording was captured as `browser-recording-mta5qsob`, but its artifact was not readable from the workspace.
- **State:** initial editor state with one wood voxel at `(0, 0, 0)`

## Findings

- [Blocked] Browser-rendered comparison evidence is unavailable.
  - Location: collaborative preview, `/game/shipwright/`.
  - Evidence: navigation and canvas initialization succeeded; snapshot failed with preview automation execution/timeout errors, and preview click/key automation also failed at the client.
  - Impact: typography, spacing, color, asset fidelity, copy, and interaction state cannot be signed off from browser-rendered evidence.
  - Fix: recapture when T3 preview canvas capture is functioning, then compare the source and implementation at the same 1280 × 720 state.

## Code-level checks completed

- The navy, brass, parchment, and violet design tokens are carried into the Bevy UI.
- Major source regions are preserved: top document bar, left block catalog, central viewport, right status inspector, bottom contextual help.
- The requested background intentionally replaces source scenery with a CSS gradient.
- The requested zero-level treatment is a transparent minor/major grid.
- Source component thumbnails intentionally become five raw block materials: wood, stone, grass, iron, and glass.
- Copy reflects raw voxel construction and current controls.
- No raster assets are required in the editor viewport; the source ship and environment imagery are intentionally replaced by the live voxel scene.

## Interaction verification

- Unit test: a new world contains exactly one centered wood voxel — passed.
- Unit test: the editor ray selects the correct exposed face of the centered voxel — passed.
- Add/remove, material buttons, keyboard material selection, orbit, zoom, undo/redo, and reset compile in native and WASM builds.
- Browser canvas initialized successfully.
- Pointer and keyboard automation in the collaborative preview was blocked by the same preview client failure, so end-to-end interaction remains unverified.

## Comparison history

### Pass 1

- Found from code review: orbit drag could place a voxel on release.
- Fix: track drag distance and only treat a release as placement below a four-pixel threshold.
- Found from code review: generic button styling could overwrite the violet Capture treatment.
- Fix: scope material-button styling to material controls only.
- Found from browser inspection: an opaque clear color prevented the requested CSS gradient from showing.
- Fix: use a transparent Bevy window/camera clear so the page gradient is visible beneath the canvas.
- Post-fix visual evidence: blocked by collaborative preview snapshot failure.

## Follow-up polish

- Replace the default system font with a bundled display/UI font pair after visual capture establishes exact metrics.
- Add icon-library glyphs to the top actions only after the base layout is visually signed off.
- Implement Capture only when image export enters scope; it is currently visual chrome.

## Final result

final result: blocked

The implementation builds and focused logic tests pass, but Product Design QA cannot pass without an inspectable browser-rendered screenshot and interaction capture.
