---
title: "Aether Isles: first public experience"
status: exploring
created: 2026-08-25
updated: 2026-08-25
owners:
  - michael
related_stories: []
related_actors: []
tags:
  - aether-isles
  - mvp
  - public-release
  - experience-design
---

# Aether Isles: first public experience

## Starting point

*Aether Isles* describes a large systemic game with voxel construction, physical ships, sails, ropes, grappling, magical currents, Aether machinery, water, fire, and exploration. Building the complete game is a long undertaking, but we do not need to wait for the complete game before giving people something meaningful from its world.

Before choosing the first implementation initiative, we need to define the **first experience we want to put in someone else's hands**.

This is more useful than asking which feature should be implemented first. A feature such as rope simulation has no public meaning on its own. An experience gives it context: escaping a fall with a grapple, swinging a ship around an anchor, building a glider, or making an object worth showing to someone else.

The first experience does not have to be the opening of the final game. It may be a small game, creative tool, interactive toy, visual demonstration, or web application. It should inhabit the identity of *Aether Isles* and help us learn something useful for the larger project.

## Problem or opportunity

The complete vision is too large to remain invisible until its major systems converge. If development proceeds only through internal technical prototypes, we risk spending a long time without learning:

- what people immediately find appealing about the world;
- which actions are enjoyable without explanation;
- what visual moments people want to capture and share;
- whether the voxel, sky, rope, sail, and Aether identities feel distinctive;
- which technical work produces reusable value for the main game;
- what kind of public artifact can create an early community around the project.

The opportunity is to release a small, coherent experience quickly, share images and videos while building it, and let public reactions inform the next direction.

## Hypothesis

A narrow experience built around one strong fantasy will be more valuable than a broad but shallow miniature of the full game.

If the experience is understandable in seconds, visually identifiable as *Aether Isles*, playable or useful in a browser, and naturally produces shareable moments, then it can:

- give the project a public existence early;
- generate observations and stories from real use;
- validate part of the art direction and interaction vocabulary;
- exercise reusable game technology without requiring the whole game;
- reveal which experience deserves promotion into an initiative.

The first release should not try to prove every pillar. It should make one promise and fulfill it well.

## Exploration

### What should qualify as the first experience?

The first experience should ideally:

- be explainable in one sentence;
- become interesting within the first minute;
- work through a link in a browser;
- also run natively when development or performance requires it;
- have a recognizable *Aether Isles* visual identity;
- create good screenshots or short video clips without staging;
- offer a reason to replay, experiment, create, or share;
- use at least one piece of technology that can inform the larger game;
- remain small enough to finish and polish;
- avoid depending on the complete voxel engine or world simulation unless that dependency is the point of the experience.

“Quickly” still needs a concrete constraint. A useful starting assumption is that the first public artifact should be small enough to reach a presentable release in weeks rather than months. That assumption must be tested against candidate scope and available agent capacity.

### Experience ideas

These are starting directions, not selected solutions.

#### 1. Skyship builder and gallery

A browser tool where people assemble a stylized skyship from a constrained set of blocks and components, pose it in an atmospheric scene, and export an image or shareable design.

Possible experience:

> Build a strange little skyship, discover how its silhouette changes as it grows, then share it against the clouds.

Potential extensions include exporting a 3D model or a version prepared for 3D printing. The first release would not need physical simulation; construction quality and sharing could carry the experience.

What it could teach us:

- construction interaction;
- visual language for blocks and ship components;
- camera and presentation tooling;
- which shapes people enjoy building;
- serialization and shareable designs;
- early voxel or modular meshing requirements.

#### 2. Glider descent

A short traversal game in which the player glides downward through a vertical archipelago, choosing routes through rings, waterfalls, caves, ruins, and magical wind.

Possible experience:

> Dive from a floating island and find a beautiful route through the sky before your momentum runs out.

It could be score-based, endless, daily-seeded, or built around a short authored descent. It would express verticality and movement without requiring ship construction.

What it could teach us:

- first-person or third-person aerial movement;
- readable wind and current visualization;
- atmosphere, scale, and distant islands;
- glider handling;
- procedural or authored route composition;
- replay and score-sharing behavior.

#### 3. Rope-run escape

A compact momentum game combining running, jumping, a wearable grapple, swinging, and perhaps a small glider. The player moves through collapsing sky ruins or along the outside of a moving vessel.

Possible experience:

> Run across impossible structures, catch anchors with your grapple, and preserve enough momentum to escape the storm.

This could resemble an endless runner, a short time-trial course, or a sequence of handcrafted challenges.

What it could teach us:

- character movement;
- grapple readability and feel;
- rope presentation;
- momentum preservation;
- anchor language;
- short-session browser controls.

#### 4. Mainstream slingshot

A focused navigation game built around the signature ship maneuver: enter a current, grapple a fixed anchor, swing, and release into the correct branch.

Possible experience:

> Swing a tiny skyship through a luminous current and release at exactly the right moment.

The ship could be predefined. Each level would be a navigational puzzle or score challenge rather than a construction sandbox.

What it could teach us:

- whether the signature maneuver is fun;
- force-field and current readability;
- grapple and rope tension;
- ship-scale motion;
- level design around three-dimensional trajectories;
- the visual identity of the Grand Mainstream.

This is highly representative of the final game, but technically riskier than a builder or simple glider experience.

#### 5. Aether pipe puzzle

A small engineering puzzle where the player connects collectors, tanks, pumps, valves, and machines using visible purple Aether.

Possible experience:

> Route luminous liquid magic through a tiny machine and watch the whole construction come alive.

It could be spatial and three-dimensional without needing a voxel world. Puzzles could ask the player to balance pressure, flow, storage, or competing consumers.

What it could teach us:

- whether Aether is legible and satisfying as a physical energy system;
- component and connection UX;
- pipe rendering;
- simulation visualization;
- engineering puzzle language;
- how much complexity players enjoy.

#### 6. Sail and wind toy

An atmospheric interactive scene where players arrange masts, sails, ropes, and fans or wind sources, then watch and tune the resulting motion.

Possible experience:

> Rig a strange sail machine and discover what the magical wind makes it do.

The goal could be open-ended creation, reaching a target, lifting a weight, or propelling a small platform.

What it could teach us:

- fabric style and motion;
- constraint stability in the browser;
- sail force readability;
- rigging interaction;
- whether experimentation is satisfying without survival or progression.

#### 7. Floating-island postcard maker

A visual composition tool where people arrange islands, clouds, waterfalls, currents, ships, lighting, and a camera to create an *Aether Isles* scene.

Possible experience:

> Compose a tiny floating world and export it as a fantasy postcard.

This could deliver public images quickly and help develop the art direction before difficult simulation work.

What it could teach us:

- atmosphere and scale;
- island silhouettes;
- composition controls;
- asset and biome language;
- which visual elements make the world recognizable;
- what people choose to share.

Its principal risk is that it validates presentation more than gameplay.

#### 8. Waterfall collector

A small flying challenge where the player steers a simple vessel beneath floating islands to collect water from waterfalls without colliding or being pushed away by wind.

Possible experience:

> Fly beneath an island, hold position in the wind, and catch a waterfall in your ship's tank.

What it could teach us:

- ship handling;
- vertical spatial judgment;
- waterfalls and atmosphere;
- visible storage feedback;
- environmental resource collection;
- the relationship between sails and precise powered movement.

#### 9. Tiny skyship survival event

A short scenario aboard a predefined ship: a storm arrives, a pipe leaks, a sail tears, and the player must move around the vessel making repairs long enough to reach shelter.

Possible experience:

> Keep a fragile skyship alive for five minutes while wind and cascading failures pull it apart.

What it could teach us:

- the ship-as-home fantasy;
- readable systemic failures;
- repair interaction;
- environmental storytelling;
- which emergencies are exciting rather than frustrating.

This expresses the final vision strongly but combines too many immature systems for the earliest release unless heavily scripted.

#### 10. Shareable physics spectacles

Instead of beginning with a conventional game, release a sequence of small interactive scenes designed for capture:

- a giant sail filling with magical wind;
- a ship swinging around a fixed anchor;
- purple Aether flowing through a transparent machine;
- a burning rope snapping and releasing a structure;
- a waterfall spilling from a floating island;
- a glider passing through a cloud canyon.

Each scene could be a separate lab accessible from the public lobby. Together they would establish a cadence of public progress without pretending to be a complete game.

What it could teach us:

- which spectacle attracts attention;
- which prototype people ask to play;
- how well the web lab architecture supports frequent releases;
- which visual and physical signature should receive deeper investment.

### Possible selection dimensions

Candidate experiences should eventually be compared using the same dimensions:

| Dimension | Question |
| --- | --- |
| Immediate appeal | Can someone understand why it is interesting from a short clip or sentence? |
| Time to first delight | How quickly does a new visitor see or do something satisfying? |
| Shareability | Does normal use produce images, clips, scores, or creations worth sharing? |
| Aether identity | Does it look and feel specific to this world? |
| Technical reuse | What knowledge or code transfers to the main game? |
| Technical risk | How many uncertain systems must work simultaneously? |
| Browser suitability | Can it load quickly and run acceptably on ordinary hardware? |
| Polish surface | How much content and UI must be polished before it feels intentional? |
| Replay or creation value | Why would someone return after the first minute? |
| Public feedback value | What important project uncertainty could reactions resolve? |

These dimensions should guide discussion, but we should not invent numerical scores without evidence. A lightweight prototype or visual mockup may be more informative than a detailed speculative matrix.

### A promising portfolio approach

The choice may not need to produce one permanent MVP. We could treat the first public period as a sequence:

1. publish visual and interactive micro-experiences;
2. observe which fantasy generates the strongest reaction;
3. expand one into a small coherent game or tool;
4. promote that validated direction into an initiative;
5. continue exposing reusable labs as public artifacts when they are understandable and polished enough.

This approach fits development by multiple AI-agent teams. Independent teams could explore a builder, movement toy, physical spectacle, and art scene in parallel, provided each exploration remains tightly bounded and uses common browser and presentation infrastructure.

The risk is fragmentation: several unfinished experiments could create less value than one polished experience. Parallel exploration therefore needs a short comparison window and an explicit selection decision.

## Assumptions

- A browser link substantially lowers the barrier to trying and sharing the first experience.
- Public progress will be more useful if it demonstrates an understandable experience rather than isolated technology.
- A small side project can contribute reusable code or design knowledge without matching the final game's structure exactly.
- The world has enough visual identity to be recognizable before its full simulation exists.
- Images and short videos are valid early public outputs, even before a playable release.
- We can constrain the first experience tightly enough to polish it within weeks.
- Early public feedback will help choose later investments rather than merely reward visual novelty.

## Evidence

### Supporting

- The current project already builds and serves a Bevy game in the browser, so browser distribution has passed an initial technical smoke test.
- The game brief contains several fantasies that can stand alone: building a ship, gliding through vertical space, grappling an anchor, routing Aether, and surviving a ship failure.
- The existing lobby structure can host multiple small games or experiments under stable URLs.

### Contradicting

- We do not yet have observations from external players showing which experience they value.
- The load size and performance cost of a Bevy browser release may limit casual sharing and must be measured on real devices and connections.
- A side experience may attract interest for a mechanic that does not transfer cleanly to the larger game.
- Publishing several loosely related experiments could make the project's identity unclear.

## Open questions

- Who is the first intended audience: voxel builders, movement-game players, engineering-game players, visual-art followers, or general fantasy-game players?
- What is the maximum acceptable time to reach the first public release?
- Is the first desired public artifact playable, creative, watchable, or some combination?
- Should the experience favor learning about gameplay or establishing the visual identity?
- Which idea can create an appealing ten-second video before it becomes a complete product?
- Which idea remains enjoyable with a deliberately narrow content set?
- Do we want people to share a score, a creation, a screenshot, a replay, or simply the link?
- How small must the browser download become for frictionless sharing?
- What hardware and browser baseline should the first experience support?
- How will public reactions be recorded as stories rather than lost in transient conversations?
- How many parallel explorations can we support without delaying selection and polish?
- What would cause us to stop an exploration before it becomes an initiative?

## Next experiment

Reduce uncertainty through **experience pitches**, not technical prototypes.

For four to six leading ideas, create a one-page experience card containing:

- the one-sentence promise;
- what the visitor does in the first minute;
- the moment most likely to become a screenshot or short clip;
- the reason to replay, create, or share;
- a rough visual sketch or storyboard;
- the smallest credible public version;
- the largest technical uncertainty;
- what we would learn from releasing it.

Then select two or three contrasting ideas for very small proof artifacts. These may be mockups, animations, videos, or interactive greyboxes. Compare them using real reactions and the selection dimensions above before creating an implementation initiative.

An initial contrasting set could be:

- **Skyship builder and gallery** — creation and sharing;
- **Glider descent or rope-run escape** — immediate movement and replay;
- **Mainstream slingshot or sail-and-wind toy** — signature physical interaction;
- **Floating-island postcard maker** — visual identity and rapid public imagery.

## Decision log

| Date | Decision | Reason |
| --- | --- | --- |
| 2026-08-25 | Initial exploration created | Define the first public experience before selecting implementation work |
| 2026-08-25 | Frame the MVP as an experience rather than a feature subset | A coherent promise can be evaluated and shared; an isolated feature cannot |
| 2026-08-25 | Keep side projects in scope | A small game, tool, or visual experience may deliver learning and public value sooner than the opening of the final game |
