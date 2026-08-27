# Aether Isles

A Bevy workspace for the game described in `BRIEF.md`. Gameplay capabilities live in reusable library crates; executable games only compose plugins and add scenario-specific setup.

## Prerequisites

- Current stable Rust, including the `wasm32-unknown-unknown` target
- `wasm-bindgen-cli` matching the version in `Cargo.lock` for browser packaging

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.127 --locked
```

## Native

Build every executable so the lobby can find them, then run the lobby:

```sh
cargo build --workspace
cargo lobby
```

Run the game directly with:

```sh
cargo runner
cargo sandbox
```

Run the voxel ship editor directly with:

```sh
cargo shipwright
```

## Browser

```sh
./scripts/build-web.sh
python3 -m http.server --directory dist 8080
```

Open `http://localhost:8080/`. The sandbox has the stable route:

```text
/game/sandbox/
```

The voxel ship editor has the stable route:

```text
/game/shipwright/
```

Controls: WASD to move, Space/left Shift to move vertically, and hold the left mouse button while moving the mouse to look.
The lane runner is available at `/game/runner/`.

Runner controls: Left/Right Arrow or A/D changes lane and Space jumps. While airborne, Space toggles the glider on or off. Jump near a glowing anchor and hold F or E to attach the rope; release to launch with your swing momentum. Mobile supports horizontal swipes, swipe-up for jump/glide toggle, and touch-and-hold for the rope. Sandbox controls remain WASD, Space/left Shift, and hold-left-mouse look.

Shipwright controls: click an exposed voxel face to add the selected material, Shift-click to remove, drag to orbit, scroll to zoom, use 1–5 to select wood/stone/grass/iron/glass, and Ctrl/Cmd+Z to undo.

With the web server running, execute the browser smoke test with:

```sh
npm install
npm run test:browser
```

## Adding a game

1. Add its binary crate under `games/` and register it in the workspace.
2. Compose shared feature plugins in its `main.rs`.
3. Add it to both lobby catalogs.
4. Add its WASM package and `/game/<name>/` output to `scripts/build-web.sh`.

The duplicated lobby catalog is deliberately tiny for now. Once several games exist it should move to a shared data file consumed during both builds.


## Repository structure

This repository is also the working knowledge base for the business:

```text
aether/
├── initiatives/  Active directions, specifications, and delivery tracking
├── stories/     Factual observations and experiences from the field
└── thoughts/    Explorations, hypotheses, and possible solutions
```

Each area has its own guide and template. Use lowercase kebab-case names so folders and files remain easy to search and automate.

- A **story** records what happened or what was observed. It should distinguish evidence from interpretation.
- A **thought** explores what something could mean, how a problem might be solved, or what might happen next.
- An **initiative** is a direction chosen for active work. It defines a measurable outcome and follows the work through implementation and iteration.
- An **actor** records what is known about a real person or organization and the relationship with them.

A story can inspire one or more thoughts. A validated thought can become an initiative, and an initiative's releases can produce new stories and corrections. Any of these may identify actors to interview or serve. Link related records rather than duplicating their contents.
