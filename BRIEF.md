# Aether Isles

## Game Concept & Systems Specification

**Status:** Concept / Pre-production
**Genre:** 3D voxel sandbox survival, exploration and engineering adventure
**Core fantasy:** Build a machine that lets you survive, navigate and eventually master a vast three-dimensional ocean of floating islands, magical currents and living physical systems.

---

# 1. High Concept

*Aether Isles* is a 3D voxel sandbox set in a vast breathable sky suspended in the void.

Countless floating islands exist at radically different altitudes. They are connected not by an ocean, but by enormous three-dimensional magical currents. The largest of these is known as **the Grand Mainstream**.

Players begin with very little, harvesting resources from an island and constructing increasingly sophisticated tools, structures and vehicles.

The defining progression is:

**survive → explore → harvest → build → sail → engineer → master the currents → reshape the world**

Ships are not predefined vehicles. They are voxel constructions assembled by the player from structural blocks, sails, ropes, engines, tanks, pumps, balloons, propellers, grapples and other components.

The world itself is highly physical:

- islands are independent voxel entities;
- ships are independent voxel entities;
- voxel structures can break and detach;
- water flows;
- sails react to wind;
- ropes transmit tension;
- grapples constrain moving voxel entities;
- fire destroys structures;
- wet fabric becomes heavy;
- Aether circulates through visible pipes;
- currents physically move ships through space.

The goal is to make the world feel like a **giant systemic construction toy embedded inside an adventure game**.

---

# 2. Game Pillars

## 2.1 Voxel Engineering

Almost everything is constructed from voxels.

The player should be able to look at a machine and understand how it works physically.

A ship is not a vehicle prefab containing invisible statistics.

Its performance emerges from what the player built:

- hull;
- mass;
- sails;
- engines;
- tanks;
- propellers;
- balloons;
- ballast;
- ropes;
- pulleys;
- grapples;
- structural supports.

The player's creations can range from crude functional machines to enormous flying cities.

---

## 2.2 A Living Physical World

Physics should create gameplay rather than merely visual effects.

Examples:

- a sail catches magical wind;
- a rope becomes tense;
- a grappling cable swings a ship around a fixed anchor;
- water moves toward a hole;
- a damaged pipe leaks Aether;
- wet fabric sags;
- fire burns through a rope;
- losing one sail changes ship handling;
- a detached piece of a structure becomes its own physical object.

The simulation does not need to reproduce reality perfectly.

It needs to be:

**readable, predictable, expressive and fun.**

---

## 2.3 The Ship Is the Player's Home

The player's primary long-term creation is their ship.

A first vessel may be little more than:

- a platform;
- a small sail;
- a rudder;
- a few storage blocks.

Over time it can evolve into:

- a sailing ship;
- an Aether-powered vessel;
- a cargo hauler;
- a workshop;
- an exploration vessel;
- an airship;
- a mobile factory;
- a flying fortress;
- an entire travelling settlement.

Ideally, parts of the player's primitive original vessel remain physically embedded in the gigantic machine they build dozens of hours later.

---

## 2.4 Exploration Through Movement

Travel should itself be gameplay.

Players do not simply select another island on a map.

They must understand:

- wind;
- starlight;
- currents;
- altitude;
- branches of the Grand Mainstream;
- portals;
- fixed anchors;
- ship momentum;
- available energy.

Mastering navigation should feel comparable to learning how to sail an ocean.

---

## 2.5 Strong Voxel Identity

The game must remain visually voxel-based even when systems deform.

Water, fabric, ropes and destruction should not suddenly become conventional smooth meshes that look disconnected from the world.

The goal is:

**voxel materials behaving in surprising ways without losing their voxel readability.**

---

# 3. The Universe

The universe is mostly void.

Inside it exist gigantic regions filled with a **breathable atmospheric medium**, containing clouds, weather, magical wind and floating islands.

From a great distance, these regions could appear almost like gaseous oceans suspended in space.

There is no universal flat "world surface."

Everything exists in true 3D.

An island might be:

- hundreds of metres above another;
- kilometres below a city;
- upside-down relative to another formation if local world rules eventually permit it;
- hidden inside cloud layers;
- positioned beside a vertical branch of the Mainstream.

The player should regularly look upward and downward and see distant islands.

The world should communicate enormous vertical scale.

---

# 4. Floating Islands

Each island is an independent **voxel entity**.

An island may contain:

- terrain;
- caves;
- vegetation;
- minerals;
- water reservoirs;
- ruins;
- settlements;
- machinery;
- creatures;
- Aether crystals.

Possible island scales range from tiny floating rocks to enormous regions several kilometres wide.

Islands can support distinct biomes, cultures and architectural periods.

Because islands are separate objects rather than part of one infinite terrain grid, they can also form natural units for:

- streaming;
- loading and unloading;
- simulation LOD;
- world generation;
- destruction isolation.

---

# 5. The Grand Mainstream

## 5.1 Concept

The **Grand Mainstream** is an enormous persistent current of magical wind and Aether flowing through the world.

It is not flat.

It twists through three-dimensional space:

- horizontally;
- vertically;
- diagonally;
- around islands;
- through cloud layers;
- into spirals;
- across gigantic altitude differences.

Seen from afar, it resembles a luminous celestial river winding through the sky.

---

## 5.2 Forced Movement

Once an unconstrained voxel entity enters the strong core of the Mainstream, it is carried with it.

This applies to:

- ships;
- players;
- debris;
- detached voxel fragments;
- possibly creatures.

Powerful engines may influence orientation and local movement but cannot simply negate the strongest current.

This makes the Mainstream both:

- a transportation network;
- a navigational hazard.

---

## 5.3 Branches

The Mainstream periodically divides into branches.

A branch can lead toward:

- another island group;
- another altitude;
- a dangerous biome;
- a trading region;
- ruins;
- a portal;
- a hidden route.

Players must position themselves correctly before a junction.

Missing a branch may carry the ship far away.

---

## 5.4 Swirl Exits

Some branches form large rotational currents.

A ship may need to:

1. enter the swirl;
2. build angular momentum;
3. grapple an anchor;
4. change its trajectory;
5. release at the correct moment;
6. launch into an exit stream.

Navigation becomes partly about reading large-scale fluid motion.

---

# 6. Fixed Space Anchors

Certain rare magical gems can become fixed relative to space.

Civilizations use them to construct **immovable anchors**.

These anchors remain stationary even inside powerful currents.

They can support:

- navigation;
- harbours;
- buildings;
- gates;
- rescue systems;
- grappling manoeuvres.

Their existence is extremely important to the world's technology.

An anchor provides something almost nothing else in this universe can provide:

> an absolute point against which force can be applied.

---

# 7. Portals

Some Mainstream exits are controlled by structures protected or stabilised by special gems.

These can form portals or gates allowing ships to escape a current or transition toward another region.

Portals may therefore act as:

- shortcuts;
- infrastructure;
- strategic locations;
- trade routes;
- progression gates;
- ancient technology.

---

# 8. Grappling

Grappling is a major traversal and navigation system.

## 8.1 Wearable Grapple

Players can carry a personal grappling device.

It allows them to:

- cross gaps;
- escape a current;
- attach to ships;
- board structures;
- catch fixed anchors;
- swing around terrain;
- recover after falling.

Movement should preserve momentum.

A strong current pulling the player while a cable is attached should create visible cable tension and a dramatic swing.

---

## 8.2 Ship Harpoon Blocks

Large grappling blocks can be built onto voxel ships.

They launch heavy harpoons connected by rope or cable.

Possible targets include:

- fixed anchors;
- islands;
- another ship;
- floating debris;
- structures;
- large movable voxel entities.

A harpoon creates a physical constraint.

---

## 8.3 Ship Swinging

One of the signature manoeuvres of the game:

1. a ship is travelling rapidly inside the Mainstream;
2. the player fires a harpoon at a fixed anchor;
3. the cable becomes violently tense;
4. the ship continues moving with the current;
5. tension redirects its trajectory;
6. the entire voxel ship swings around the anchor;
7. the crew releases the line at the correct moment;
8. the ship enters an exit branch.

The player should **feel the tension in the cable and mass of the ship**.

This should be one of the game's most spectacular mechanical interactions.

---

# 9. Voxel Entities

The world is composed of multiple independent voxel bodies rather than a single global voxel grid.

Examples:

- island;
- ship;
- detached structure;
- debris;
- moving machine;
- possibly large creatures.

A voxel entity contains its own voxel coordinate system.

This allows a ship to:

- translate;
- rotate;
- collide;
- break;
- potentially split into multiple entities.

---

# 10. Structural Destruction

Structures can lose connections.

If a section becomes structurally disconnected, it can detach and become a new voxel entity.

Examples:

- a mast snaps from a ship;
- a bridge collapses;
- part of an island breaks away;
- a damaged cargo section falls from a vessel;
- burning architecture collapses.

Destruction should therefore affect actual gameplay systems rather than being purely cosmetic.

---

# 11. Ships

Ships are constructed freely from blocks.

There is no mandatory hull shape.

Possible components include:

- hull voxels;
- beams;
- decks;
- masts;
- sails;
- balloons;
- propellers;
- rudders;
- engines;
- Aether tanks;
- pipes;
- pumps;
- anchors;
- grapples;
- winches;
- cranes;
- landing gear;
- ballast tanks;
- storage;
- workshops.

The simulation determines how the resulting construction behaves.

Players should be able to create strange designs that genuinely work if their physics are sound.

---

# 12. Sailing

Sails are an important source of propulsion.

They can interact with:

- ordinary atmospheric wind;
- magical winds;
- the Grand Mainstream;
- potentially starlight.

Sails provide free propulsion but depend heavily on environmental conditions.

Engines provide precision and reliability but consume stored energy.

This produces a natural relationship:

**sails for efficient travel; engines for control.**

A good vessel is likely to combine both.

---

# 13. Magical Starlight

Stars are not only visual objects.

Their light carries usable magical energy.

Special crystals can capture it.

Different celestial conditions could influence:

- available energy;
- sail propulsion;
- crystal efficiency;
- creatures;
- vegetation;
- environmental effects.

Large islands can create actual **energy shadows**.

A ship entering the shadow of a huge island may suddenly stop generating starlight energy and need to rely on its stored reserves.

---

# 14. Magical Wind

Magical winds move throughout atmospheric regions.

They can:

- propel sails;
- power turbines;
- form currents;
- transport particles;
- shape weather.

Their direction can be communicated visually using:

- clouds;
- vegetation;
- flags;
- particles;
- luminous trails;
- instruments;
- sail deformation.

---

# 15. Aether

## 15.1 Concept

Aether is a luminous purple magical liquid.

Functionally it occupies the role that electricity, hydraulic fluid and mana normally occupy separately.

It powers machines.

It is deliberately physical and visible.

---

## 15.2 Capture

Aether can be generated or captured from natural energy sources using crystals.

Major sources include:

- starlight;
- magical wind.

A collector converts environmental energy into usable Aether.

---

## 15.3 Storage

Aether is stored in tanks.

Tanks are effectively batteries, but because the energy is a liquid their state is visible to the player.

A player can literally see how much energy remains.

---

## 15.4 Distribution

Aether moves through pipes.

Possible components:

- collector;
- pump;
- tank;
- pipe;
- valve;
- regulator;
- machine.

Example:

**Starlight crystal → collector → pump → tank → distribution pipes → motor**

---

## 15.5 Aether Variables

The system can expose three intuitive concepts:

### Volume

How much Aether exists.

### Pressure

How strongly it is pressurised.

### Flow

How quickly it moves through a pipe.

This allows deeper engineering without requiring electronics simulation.

For example:

- a pipe may have insufficient flow;
- an engine may require minimum pressure;
- a pump can increase pressure;
- a damaged pipe leaks;
- an overloaded pipe may rupture.

---

# 16. Machines

Aether can power:

- motors;
- propellers;
- pumps;
- cranes;
- winches;
- grapples;
- tools;
- industrial machines;
- doors;
- defensive systems;
- automated systems.

Machines should visibly connect into the ship's physical infrastructure.

---

# 17. Voxel Fabric

Fabric is represented using a special **2D voxel structure simulated in 3D**.

It must remain visually voxel-based rather than becoming a smooth cloth mesh.

## 17.1 Preferred Model

Each visible fabric element behaves like a small rigid square.

The square itself does not deform.

Neighbouring tiles are connected through flexible constraints.

The result resembles:

> rigid square plates stitched together using elastic connections.

The underlying solver can use a particle/constraint lattice rather than treating every tile as a complete rigid physics body.

---

## 17.2 Fabric Properties

Fabric materials can vary through:

- mass;
- stretching;
- shear resistance;
- bending resistance;
- aerodynamic drag;
- lift;
- tear strength;
- water absorption;
- fire resistance.

This permits multiple behaviours from the same system.

---

# 18. Fabric Use Cases

The same underlying system supports:

## Sails

Catch wind and generate ship force.

## Flags

React strongly to wind and communicate environmental movement.

## Building Ornaments

Banners, awnings, curtains, suspended decorations and canopies make settlements visibly alive.

## Trampolines

Stretched fabric stores and releases mechanical energy.

## Balloons

Closed or semi-closed fabric surfaces form buoyant envelopes.

## Tents and Shelters

Flexible construction assembled using poles, ropes and fabric.

---

# 19. Rope System

Ropes use essentially the same physics concept as fabric but reduced to a **1D grid**.

A rope consists of linked elements:

**voxel → voxel → voxel → voxel**

The rope can:

- bend;
- swing;
- transmit tension;
- stretch slightly;
- break.

It retains the same stylised voxel articulation as the fabric system.

---

# 20. Ropes, Pulleys and Rigid Blocks

Flexible and rigid systems can be combined.

Example sail rigging:

**rigid mast → pulley → rope → fabric sail → rope → winch → hull**

Rigid poles provide structure.

Ropes transmit forces.

Pulleys redirect force.

Winches change rope length.

Fabric reacts to the resulting tension.

This allows players to construct actual mechanical systems rather than placing abstract "working sail blocks."

The same principles can support:

- cranes;
- elevators;
- bridges;
- cargo hoists;
- traps;
- suspended structures;
- steering systems.

---

# 21. Voxel Water

Water has two related components:

1. underlying fluid behaviour;
2. a distinctive voxel surface representation.

---

# 22. Rigid-Tile Water Surface

The water surface can use a dense grid of approximately **10 cm transparent blue tiles**.

Each tile is a rigid planar element.

A tile may:

- move upward;
- move downward;
- rotate.

But:

**all four corners of a tile always remain coplanar.**

The tile never bends.

Thousands of these small articulated tiles together create the wave surface.

This creates a distinctive appearance where an ocean looks smooth from a distance but retains a subtle faceted voxel structure nearby.

---

## 22.1 Water Rendering

Tiles can use:

- transparency;
- depth colouring;
- reflection;
- refraction;
- bright specular highlights;
- foam;
- visible seams or subtle voxel borders.

The world beneath the surface should remain partially visible.

---

## 22.2 Water Interaction

Surface disturbance can propagate through neighbouring tiles.

Possible effects:

- waves;
- ripples;
- ship wakes;
- impacts;
- wind;
- waterfalls;
- rain.

The surface representation does not necessarily need to be identical to the underlying water-volume solver.

**Simulation and rendering can use different resolutions.**

---

# 23. Dynamic Water

Water is a genuine resource and world system.

Possible behaviours:

- fills depressions;
- flows through openings;
- forms lakes;
- moves through pipes;
- falls from islands;
- floods structures;
- leaks from damaged tanks.

Floating islands can contain underground reservoirs.

Breaking into one may flood a cave.

---

# 24. Water Gameplay

Water can support:

- drinking;
- farming;
- irrigation;
- cooling;
- firefighting;
- ballast;
- industrial processes;
- reservoirs;
- aqueducts.

Ships can collect water directly from waterfalls while flying beneath islands.

---

# 25. Wet Materials

Fabric and some other materials can absorb water.

Wet fabric becomes:

- heavier;
- less responsive;
- more sagging;
- darker;
- potentially less flammable.

Gameplay examples:

- a wet sail produces less useful lift;
- a trampoline has reduced bounce;
- a balloon becomes heavier;
- banners sag after a storm.

Wetness should affect both **rendering and physics**.

---

# 26. Fire

Fire propagates through flammable materials.

Possible affected materials:

- wood;
- fabric;
- rope;
- vegetation.

Fire changes both appearance and structure.

Fabric can:

1. ignite;
2. darken;
3. weaken;
4. develop holes;
5. lose connections;
6. tear;
7. detach.

A burning rope may snap suddenly and release the load it was holding.

A sail losing material provides less propulsion.

Fire on a ship can therefore create cascading mechanical failures.

---

# 27. Damage and Material State

World materials can respond to multiple conditions.

Example fabric state:

**healthy → stressed → torn → detached**

Wet state:

**dry → damp → soaked**

Fire state:

**normal → burning → charred → destroyed**

These states should modify:

- visuals;
- mass;
- stiffness;
- strength;
- drag;
- lift;
- connection integrity.

---

# 28. Survival

Survival should exist but should not overwhelm the engineering adventure.

Potential survival concerns:

- water;
- shelter;
- temperature;
- injuries;
- environmental exposure;
- exhaustion.

Food can exist without becoming a constantly draining timer that interrupts exploration.

The goal is closer to an adventure survival system than a hardcore survival simulator.

---

# 29. Resources

The player gathers resources from different islands.

Core categories may include:

- wood;
- stone;
- metal;
- fibres;
- crystals;
- water;
- rare magical materials.

Different biomes provide different combinations.

This creates reasons to travel rather than remain permanently on one island.

---

# 30. Progression

## Phase 1 — Stranded

The player starts on an island with primitive tools.

Primary goals:

- survive;
- gather;
- craft;
- build shelter.

Nearby islands are visible but difficult to reach.

---

## Phase 2 — Glider

The player constructs simple fabric and rigid-frame gliders.

Nearby islands become accessible.

Vertical exploration begins.

---

## Phase 3 — Sailing Platform

The player discovers enough technology to create a small sailing vessel.

The world suddenly becomes much larger.

---

## Phase 4 — Powered Ship

Aether systems unlock:

- collectors;
- tanks;
- pumps;
- pipes;
- motors.

The player is no longer completely dependent on favourable wind.

---

## Phase 5 — Industrial Ship

Advanced systems enable:

- cranes;
- processing;
- automation;
- large storage;
- sophisticated rigging;
- heavy propulsion.

The ship becomes a mobile base.

---

## Phase 6 — Mainstream Mastery

The player learns to navigate dangerous high-energy currents.

Advanced travel requires:

- grappling;
- anchors;
- current reading;
- portals;
- branch timing;
- energy management.

---

## Phase 7 — Megastructures

Late-game players can construct enormous ships, settlements and infrastructure.

Possible future capabilities include:

- linking islands;
- moving island fragments;
- building sky ports;
- controlling local currents;
- creating artificial anchors;
- manipulating Aether at massive scale.

---

# 31. Core Gameplay Loop

The fundamental loop is:

**Explore → Discover → Harvest → Transport → Build → Engineer → Upgrade → Travel Farther**

A secondary loop occurs aboard the ship:

**Observe environment → Configure sails/engines → Navigate → React to danger → Repair → Continue**

And an engineering loop:

**Build → Test → Observe failure → Modify → Improve**

Failure should often create knowledge rather than simply punishment.

---

# 32. World Diversity

The setting should not feel culturally or historically uniform.

Different island groups can represent radically different societies.

Variation can include:

- architecture;
- clothing;
- technology;
- materials;
- ship design;
- settlement organisation;
- cultural traditions;
- relationship with Aether.

Human characters should represent broad ethnic diversity.

The world can also contain non-human peoples if appropriate later.

---

# 33. Architectural Diversity

Possible visual families include:

- ancient monumental stone cities;
- overgrown temple cultures;
- timber sky villages;
- ornate merchant ports;
- nomadic fabric settlements;
- industrial steampunk cities;
- fortress societies;
- crystalline magical architecture;
- frozen monasteries;
- ruins from forgotten technological eras.

These should be remixed into original cultures rather than directly reproducing individual historical societies.

---

# 34. Multiple Eras

Not every island developed at the same speed.

The world can simultaneously contain:

- ancient ruins;
- traditional sailing cultures;
- early mechanical societies;
- industrial Aether civilizations;
- highly advanced magical technology.

This creates a feeling that the world has experienced multiple cycles of development, collapse and rediscovery.

---

# 35. Biomes

Possible major biome families include:

### Verdant Islands

Dense forests, giant roots, rivers and waterfalls.

### Tropical Archipelagos

Warm oceans, lagoons, vegetation and settlements.

### Crystal Reaches

Large magical mineral formations and intense starlight interactions.

### Frozen Heights

Snow, ice, glaciers and exposed high-altitude regions.

### Ember Regions

Volcanoes, lava, industrial extraction and extreme heat.

### Storm Frontiers

Violent currents, lightning and unstable islands.

### Desert / Sunlit Routes

Dry rocky environments inhabited by traders and travelling cultures.

### Dark or Void Regions

Low light, weak starlight and unusual creatures.

Each biome should influence gameplay as well as visuals.

---

# 36. Vertical Ecology

Altitude itself acts almost like another biome dimension.

Different heights may have:

- different wind;
- different Mainstream directions;
- different temperatures;
- different cloud types;
- different creatures;
- different available starlight;
- different resources.

Navigation therefore occurs in **XYZ**, not just XY.

A player might deliberately climb several kilometres to find a favourable current.

---

# 37. World Navigation

There should not be a conventional flat map that implies every island occupies the same plane.

Navigation interfaces should communicate:

- altitude;
- depth;
- current direction;
- branches;
- islands;
- portals;
- anchors.

A possible representation is an interactive volumetric navigation chart rather than a conventional map.

---

# 38. Art Direction

The visual target combines:

- voxel geometry;
- painterly fantasy;
- expressive colour;
- readable silhouettes;
- atmospheric lighting;
- stylised animation.

The broad emotional reference is the adventurous readability and colourful atmosphere associated with modern *Zelda*, translated into a voxel universe rather than copied literally.

---

# 39. Visual Principles

## Large Shapes First

Avoid excessive voxel noise.

World silhouettes should remain immediately understandable.

## Strong Atmosphere

Use:

- clouds;
- volumetric light;
- atmospheric perspective;
- mist;
- waterfalls;
- distant islands.

These elements communicate enormous scale.

## Physical Motion Everywhere

The world should rarely appear completely static.

Players should see:

- sails moving;
- flags fluttering;
- ropes swaying;
- water rippling;
- waterfalls falling;
- clouds moving;
- distant ships travelling;
- Aether flowing through pipes.

## Aether Purple

Luminous violet Aether is the game's strongest visual signature.

It should appear in:

- tanks;
- pipes;
- engines;
- crystals;
- currents;
- portals;
- technology.

---

# 40. Scale

The game should deliberately contrast scales.

### Small

- individual 10 cm water facets;
- pipe fittings;
- fabric tiles;
- ropes;
- valves.

### Human

- player;
- workshops;
- houses;
- small boats.

### Large

- ships;
- castles;
- waterfalls;
- islands.

### Enormous

- archipelagos;
- Mainstream rivers;
- storms;
- cloud oceans;
- megastructures.

Seeing several scales at once should create much of the game's visual spectacle.

---

# 41. Desired Player Experiences

The game should repeatedly create moments such as:

- standing on an island cliff while enormous currents twist kilometres below;
- watching ships travel above and beneath you at different altitudes;
- sailing directly through a luminous 3D river in the sky;
- grappling an anchor while a ship is pulled sideways by the Mainstream;
- watching a taut rope redirect an entire vessel;
- flying underneath an island waterfall to collect water;
- repairing an Aether leak while purple liquid escapes through the hull;
- seeing a burning rope snap and release part of the rigging;
- watching a wet sail sag during a storm;
- constructing a strange machine and discovering that it actually works;
- finding a civilization whose architecture, technology and culture are radically different from anything previously encountered;
- looking back after many hours and realising that the tiny original raft has become part of an enormous skyship.

---

# 42. System Unification

A major design objective is to avoid having dozens of unrelated bespoke systems.

Several mechanics can share common abstractions.

### Voxel Bodies

Used by:

- islands;
- ships;
- debris;
- structures.

### Constraint Grids

Used by:

- 2D fabric;
- 1D rope;
- possibly water-surface articulation.

### Material States

Used by:

- wetness;
- heat;
- fire;
- damage.

### Fluid Networks

Used by:

- Aether;
- water;
- potentially other liquids.

### External Force Fields

Used by:

- wind;
- Mainstream;
- gravity;
- propulsion.

The same physical vocabulary should appear throughout the world.

---

# 43. Performance Philosophy

Simulation fidelity should be concentrated where the player can perceive it.

General strategy:

**render finely, simulate coarsely.**

Examples:

- a sail can render hundreds of voxel tiles while using a lower-resolution constraint lattice;
- distant fabric can update at lower frequency or sleep;
- water can display dense 10 cm surface facets while using a coarser underlying fluid solver;
- distant islands can unload entirely;
- only nearby fractured structures require detailed physics.

The stylised voxel aesthetic is an advantage because visually convincing behaviour does not require microscopic physical accuracy.

---

# 44. Physics Philosophy

The game should not attempt to simulate reality perfectly.

Instead, forces should produce behaviour that is:

- understandable;
- exaggerated enough to read;
- stable;
- deterministic enough for engineering;
- visually spectacular.

The player should eventually learn to predict the world.

That predictability is what transforms physics from chaos into a construction game.

---

# 45. Lore Foundation

The universe contains a substance or field connecting:

- stars;
- wind;
- currents;
- crystals;
- technology.

Civilizations discovered that certain crystals could capture and manipulate this energy.

Eventually this led to Aether technology.

Other crystals display even stranger properties, including the ability to establish points fixed in space.

These discoveries enabled:

- permanent anchors;
- sky ports;
- controlled Mainstream exits;
- portals;
- long-distance trade;
- increasingly large flying vessels.

Different civilizations interpreted and developed these discoveries in different ways.

Some worship the stars.

Some engineer them.

Some travel continuously.

Some build enormous fixed cities around ancient anchors.

Some inhabit ruins left by civilizations that understood Aether better than anyone alive today.

---

# 46. The Central Mystery

The Grand Mainstream should ultimately be more than weather.

Questions the world can gradually raise:

- Where does it originate?
- Where does it end?
- Why does it connect particular islands?
- Why are some gems capable of resisting it?
- Who created the first fixed anchors?
- Are portals natural or artificial?
- Why are ancient ruins found across radically distant regions?
- Is Aether merely energy, or something deeper?
- Are the islands themselves being carried somewhere?

The game can begin as a survival and engineering sandbox while gradually revealing a much larger cosmological mystery.

---

# 47. Tone

The universe should primarily evoke:

**wonder, freedom, discovery, ingenuity and adventure.**

Danger exists:

- storms;
- falls;
- fire;
- hostile environments;
- damaged vessels;
- uncontrolled currents.

But the world should remain inviting enough that seeing a mysterious island creates the immediate reaction:

> “I want to find out what is over there.”

---

# 48. Current Open Design Questions

The following systems remain intentionally unspecified and should be explored during prototyping:

- exact combat model;
- hostile creatures and factions;
- multiplayer versus solo structure;
- procedural versus authored island generation;
- exact gravity model;
- atmosphere boundaries;
- inventory model;
- crafting complexity;
- automation depth;
- character progression;
- death / recovery rules;
- NPC simulation;
- economy and trading;
- quests and narrative structure;
- exact late-game objective.

These should be decided only after proving the core physical sandbox.

---

# 49. Prototype Priorities

The first prototypes should prove the features that define the game.

## Prototype A — Voxel Bodies

- independent voxel island;
- independent voxel ship;
- movement and rotation;
- edit voxels;
- detach connected regions.

## Prototype B — Ship Physics

- construct a hull;
- mass;
- sail;
- propulsion;
- steering.

## Prototype C — Fabric and Rope

- rigid voxel fabric tiles;
- wind;
- attachment points;
- rope;
- pulleys;
- tearing.

## Prototype D — Grand Mainstream

- 3D current spline / volume;
- ship captured by current;
- current branches;
- swirl;
- exit manoeuvre.

## Prototype E — Grapple

- fixed anchor;
- harpoon;
- tension;
- ship swing;
- release momentum.

## Prototype F — Aether

- collector;
- tank;
- pump;
- pipe;
- motor;
- leak.

## Prototype G — Water

- local water body;
- articulated voxel surface;
- flow;
- waterfall;
- pumping.

## Prototype H — Destruction

- fire;
- wetness;
- broken rope;
- damaged fabric;
- detached voxel structures.

Together, these prototypes should answer the fundamental question:

> **Does interacting with this physical voxel world feel good enough that building and travelling are enjoyable even before quests, combat and narrative are added?**

---

# 50. One-Sentence Vision

**Aether Isles is a systemic voxel adventure where players build physical skyships, harness liquid magic, sail through three-dimensional celestial currents, and use the laws of a living destructible world to explore an endless archipelago suspended in the clouds.**