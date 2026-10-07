use crate::{
    app::Phase,
    controls::{Action, Actions},
    editor::Editor,
};
use aether_core::{
    BodyId, fixtures,
    save::{Session, TetherSave, VERSION, VesselSave},
};
use aether_sim::{self as sim, SessionEntity, Vessel};
use aether_view::{
    BodyVisual, Palette,
    environment::{self, EnvironmentVisual},
};
use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Resource)]
pub struct GameSession {
    pub expedition: aether_core::expedition::Expedition,
    pub active: Option<Entity>,
    pub walker: Option<Entity>,
    pub checkpoint: u32,
    pub progress: u32,
    pub notice: String,
    pub autosave_elapsed: f32,
    pub fuel_warning: bool,
    pub seed: u64,
    pub return_phase: Phase,
    pub settings_return_phase: Phase,
}
impl Default for GameSession {
    fn default() -> Self {
        Self {
            expedition: default(),
            active: None,
            walker: None,
            checkpoint: 0,
            progress: 0,
            notice: "Votre premier vaisseau vous attend au chantier.".into(),
            autosave_elapsed: 0.0,
            fuel_warning: false,
            seed: 7391,
            return_phase: Phase::Playing,
            settings_return_phase: Phase::Menu,
        }
    }
}
pub fn starter() -> Session {
    let body = fixtures::explorer();
    Session {
        version: VERSION,
        seed: 7391,
        tick: 0,
        active: BodyId(1),
        checkpoint: 0,
        progress: 0,
        tether: None,
        walker: None,
        expedition: default(),
        vessels: vec![VesselSave {
            circuit: None,
            id: BodyId(1),
            fuel: body.fuel_capacity(),
            blueprint: body.blueprint(),
            position: sim::ISLANDS[0].dock,
            rotation: Quat::IDENTITY,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            trim: 0.0,
            target_altitude: Some(12.0),
            docked: true,
        }],
    }
}
pub fn setup(world: &mut World) {
    replace(world, starter());
}
#[derive(Resource)]
struct EnvironmentSeed(u64);

pub fn replace(world: &mut World, session: Session) {
    if let Err(error) = session.validate() {
        world.resource_mut::<GameSession>().notice = error.to_string();
        return;
    }
    // Publish preparation immediately; don't expose a zero-length queue from the
    // previous session before the presentation systems observe the new bodies.
    world.resource_mut::<aether_view::MeshMetrics>().pending = 1;
    // Scenery is immutable for a world seed. Recreating every island and flow
    // on each load stalled the open-world browser for several seconds. Only
    // simulation/session entities are replaced; animation reads the new clock.
    let reuse_environment = world
        .get_resource::<EnvironmentSeed>()
        .is_some_and(|environment| environment.0 == session.seed);
    let mut remove: Vec<_> = world
        .query_filtered::<Entity, With<SessionEntity>>()
        .iter(world)
        .collect();
    if !reuse_environment {
        remove.extend(
            world
                .query_filtered::<Entity, With<EnvironmentVisual>>()
                .iter(world),
        );
    }
    for entity in remove {
        world.despawn(entity);
    }
    world.resource_mut::<Editor>().reset();
    if let Some(mut tutorial) = world.get_resource_mut::<crate::tutorial::Tutorial>() {
        tutorial.reset_observation();
    }
    let mut spawned = Vec::new();
    {
        let mut commands = world.commands();
        sim::spawn_world(&mut commands);
        for saved in &session.vessels {
            let entity = sim::spawn_vessel(&mut commands, saved.clone());
            let body =
                aether_core::Body::from_blueprint(saved.blueprint.clone()).expect("validated");
            commands
                .entity(entity)
                .insert((Visibility::default(), BodyVisual::new(&body)));
            spawned.push((saved.id, entity));
        }
    }
    world.flush();
    let avatar_assets = world
        .resource::<aether_view::avatar::AvatarAssets>()
        .clone();
    world.resource_scope(|world, palette: Mut<Palette>| {
        let mut commands = world.commands();
        for (_, entity) in spawned.iter().filter(|(id, _)| *id == session.active) {
            aether_view::avatar::spawn(
                &mut commands,
                &palette,
                &avatar_assets,
                *entity,
                true,
                Vec3::new(0.0, 0.25, 2.0),
            );
        }
    });
    if !reuse_environment {
        let island_scenes = world
            .resource::<aether_view::art::ArtAssets>()
            .scenes
            .clone();
        world.resource_scope(|world, palette: Mut<Palette>| {
            let mut commands = world.commands();
            for (index, island) in sim::ISLANDS.iter().enumerate() {
                commands.spawn((
                    EnvironmentVisual,
                    WorldAssetRoot(island_scenes[5 + index].clone()),
                    Transform::from_translation(island.center),
                ));
                environment::dock(&mut commands, &palette, island.dock);
            }
            for (_, position) in aether_core::world::anchors() {
                environment::anchor(&mut commands, island_scenes[9].clone(), *position);
            }
            environment::sky(&mut commands, &palette);
            commands.spawn((
                EnvironmentVisual,
                WorldAssetRoot(island_scenes[8].clone()),
                Transform::from_translation(aether_core::terrain::SANCTUARY),
            ));
        });
        world.resource_scope(|world, art: Mut<aether_view::art::ArtAssets>| {
            aether_view::art::sky(&mut world.commands(), &art);
        });
        aether_view::world::spawn(world);
        aether_view::weather::spawn(world);
        world.insert_resource(EnvironmentSeed(session.seed));
    }
    sim::fauna::spawn(
        &mut world.commands(),
        session.tick as f32 / aether_core::tuning::TICK_HZ as f32,
    );
    sim::traffic::spawn(
        &mut world.commands(),
        session.tick as f32 / aether_core::tuning::TICK_HZ as f32,
    );
    world.flush();
    let couriers: Vec<_> = world
        .query::<(Entity, &sim::traffic::Courier)>()
        .iter(world)
        .map(|(entity, courier)| (entity, courier.0))
        .collect();
    world.resource_scope(|world, art: Mut<aether_view::traffic::TrafficArt>| {
        for (entity, courier) in couriers {
            aether_view::traffic::spawn(&mut world.commands(), &art, entity, courier);
        }
    });
    let residents: Vec<_> = world
        .query::<(Entity, &sim::fauna::Resident)>()
        .iter(world)
        .map(|(e, r)| (e, r.0))
        .collect();
    for (entity, resident) in residents {
        let art =
            world.resource::<aether_view::fauna::FaunaArt>().0[resident.species as usize].clone();
        world
            .spawn((
                ChildOf(entity),
                WorldAssetRoot(art),
                aether_view::fauna::FaunaVisual(resident),
                Transform::from_translation(sim::fauna::visual_offset(resident))
                    .with_scale(Vec3::splat(resident.scale())),
            ))
            .observe(aether_view::fauna::bind);
    }
    if let Some(tether) = &session.tether {
        let anchor = world
            .query::<(Entity, &sim::Anchor)>()
            .iter(world)
            .find(|(_, a)| a.0 == tether.anchor)
            .map(|(e, _)| e);
        let vessel = spawned
            .iter()
            .find(|(id, _)| *id == tether.body)
            .map(|(_, e)| *e);
        if let (Some(vessel), Some(anchor)) = (vessel, anchor) {
            sim::attach_tether(
                &mut world.commands(),
                vessel,
                anchor,
                tether.anchor,
                tether.local_point,
                tether.length,
            );
        }
    }
    world.resource_mut::<sim::SimClock>().tick = session.tick;
    world.resource_mut::<sim::SimClock>().seed = session.seed;
    let mut game = world.resource_mut::<GameSession>();
    game.active = spawned
        .iter()
        .find(|(id, _)| *id == session.active)
        .map(|(_, e)| *e);
    game.walker = None;
    game.checkpoint = session.checkpoint;
    game.progress = session.progress;
    game.seed = session.seed;
    game.expedition = session.expedition;
    game.autosave_elapsed = 0.0;
    game.fuel_warning = false;
    if let Some(saved) = &session.walker {
        crate::controls::spawn_avatar(world, saved.position, saved.velocity);
    }
    world.resource_mut::<crate::camera::CameraRig>().snap = true;
    crate::exploration::sync_payload(world);
    world.flush();
}
pub fn snapshot(world: &mut World) -> Result<Session, String> {
    let game = world.resource::<GameSession>();
    let active = game.active.ok_or("Aucun vaisseau")?;
    let active = world.get::<Vessel>(active).ok_or("Vaisseau absent")?.id;
    let mut session = Session {
        version: VERSION,
        seed: game.seed,
        tick: world.resource::<sim::SimClock>().tick,
        active,
        checkpoint: game.checkpoint,
        progress: game.progress,
        vessels: vec![],
        expedition: game.expedition.clone(),
        tether: None,
        walker: game.walker.and_then(|e| {
            Some(aether_core::save::WalkerSave {
                position: world.get::<Position>(e)?.0,
                velocity: world.get::<LinearVelocity>(e)?.0,
            })
        }),
    };
    for (v, p, r, velocity, angular, circuit) in world
        .query::<(
            &Vessel,
            &Position,
            &Rotation,
            &LinearVelocity,
            &AngularVelocity,
            Option<&sim::aether::AetherCircuit>,
        )>()
        .iter(world)
    {
        let mut saved = sim::capture(v, p.0, r.0, velocity.0, angular.0);
        saved.circuit = circuit.map(sim::aether::AetherCircuit::capture_state);
        session.vessels.push(saved);
    }
    session.vessels.sort_by_key(|v| v.id);
    for tether in world.query::<&sim::Tether>().iter(world) {
        if let Some(v) = world.get::<Vessel>(tether.vessel) {
            session.tether = Some(TetherSave {
                body: v.id,
                anchor: tether.anchor_index,
                local_point: tether.local_point,
                length: tether.length,
            });
        }
    }
    session.validate().map_err(|e| e.to_string())?;
    Ok(session)
}
pub fn recover(world: &mut World) {
    let game = world.resource::<GameSession>();
    let checkpoint = game.checkpoint;
    let expedition = game.expedition.clone();
    let progress = game.progress;
    let seed = game.seed;
    let active = game.active;
    let Some(vessel) = active.and_then(|e| world.get::<Vessel>(e)).cloned() else {
        return;
    };
    // Reconstruct from validated construction data; recovery must work even when a
    // runtime pose has become non-finite and cannot pass save validation.
    let mut recovered = starter();
    recovered.seed = seed;
    recovered.checkpoint = checkpoint;
    recovered.progress = progress;
    recovered.expedition = expedition;
    recovered.active = vessel.id;
    recovered.vessels[0] = sim::capture(
        &vessel,
        aether_core::world::dock(checkpoint).expect("validated checkpoint"),
        aether_core::world::dock_rotation(checkpoint),
        Vec3::ZERO,
        Vec3::ZERO,
    );
    recovered.vessels[0].docked = true;
    recovered.vessels[0].fuel = vessel.body.fuel_capacity();
    if let Some(design) = vessel.body.circuit_design() {
        recovered.vessels[0].circuit = Some(
            sim::aether::AetherCircuit::new(
                &vessel.body,
                design.clone(),
                vessel.body.fuel_capacity(),
            )
            .expect("validated recovery circuit")
            .capture_state(),
        );
    }
    recovered.vessels[0].trim = 0.0;
    recovered.vessels[0].target_altitude = Some(
        aether_core::world::dock(checkpoint)
            .expect("validated checkpoint")
            .y,
    );
    replace(world, recovered);
    world.resource_mut::<GameSession>().notice =
        "Remorquage au dernier quai. Construction conservée, fragments abandonnés.".into();
    world.resource_mut::<NextState<Phase>>().set(Phase::Playing);
}
pub fn request_save(world: &mut World) {
    world.resource_mut::<Actions>().0.push_back(Action::Save);
}
