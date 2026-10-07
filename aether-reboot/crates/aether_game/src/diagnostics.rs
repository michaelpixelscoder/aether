//! Local opt-in diagnostics and integration script. No telemetry leaves the device.
use crate::{
    app::Phase,
    controls::{Action, Actions},
    editor,
    persistence::Storage,
    session::{self, GameSession},
};
use aether_core::{Block, Cell, edit::Edit};
use aether_sim::{SimClock, Vessel};
use avian3d::prelude::*;
use bevy::{platform::time::Instant, prelude::*};
use serde_json::json;
use std::collections::{BTreeMap, VecDeque};

#[derive(Resource, Default)]
pub struct Diagnostics {
    frames: VecDeque<f64>,
    ticks: VecDeque<f64>,
    render_times: BTreeMap<String, VecDeque<f64>>,
    render_last: BTreeMap<String, Instant>,
    physics_phases: BTreeMap<String, VecDeque<f64>>,
    started: Option<Instant>,
    pub enabled: bool,
    pub probe: bool,
    pub scripted: bool,
    pub output: String,
    stage: u32,
    cycles: u32,
    elapsed: f32,
    wait: f32,
    checks: Vec<String>,
    failure: Option<String>,
    pub finished: bool,
    entity_samples: Vec<u32>,
    asset_samples: Vec<[usize; 3]>,
    environment_entities: Vec<Entity>,
    published: f32,
    pub benchmark: bool,
    benchmark_started: bool,
    presentation_valid: bool,
    presentation_failure: Option<String>,
}
#[derive(Component)]
pub struct DebugText;
impl Diagnostics {
    pub fn new(scripted: bool, probe: bool, output: String) -> Self {
        Self {
            scripted,
            probe,
            output,
            ..default()
        }
    }
}
pub fn tick_start(mut diagnostics: ResMut<Diagnostics>) {
    diagnostics.started = Some(Instant::now());
}
pub fn tick_end(
    mut diagnostics: ResMut<Diagnostics>,
    collision: Res<avian3d::collision::CollisionDiagnostics>,
    solver: Res<avian3d::dynamics::solver::SolverDiagnostics>,
) {
    if let Some(started) = diagnostics.started.take() {
        diagnostics
            .ticks
            .push_back(started.elapsed().as_secs_f64() * 1000.0);
        if diagnostics.ticks.len() > 3600 {
            diagnostics.ticks.pop_front();
        }
    }
    if diagnostics.probe && diagnostics.elapsed > 5.0 {
        use avian3d::diagnostics::PhysicsDiagnostics;
        for (name, time) in collision
            .timer_paths()
            .into_iter()
            .chain(solver.timer_paths())
        {
            let values = diagnostics
                .physics_phases
                .entry(name.to_string())
                .or_default();
            values.push_back(time.as_secs_f64() * 1000.0);
            if values.len() > 3600 {
                values.pop_front();
            }
        }
    }
}
fn percentiles(values: &VecDeque<f64>) -> [f64; 3] {
    if values.is_empty() {
        return [0.0; 3];
    }
    let mut sorted: Vec<_> = values.iter().copied().collect();
    sorted.sort_by(f64::total_cmp);
    [50, 95, 99].map(|p| sorted[(sorted.len() * p / 100).min(sorted.len() - 1)])
}
fn action(world: &mut World, action: Action) {
    world.resource_mut::<Actions>().0.push_back(action);
}
fn active(world: &World) -> Entity {
    world
        .resource::<GameSession>()
        .active
        .expect("QA active session")
}
fn check(d: &mut Diagnostics, pass: bool, name: &str) -> bool {
    if pass {
        d.checks.push(name.into());
    } else {
        d.failure = Some(name.into());
    }
    pass
}
fn presentation_matches_physics(world: &mut World) -> Result<(), String> {
    let bodies: Vec<_> = world
        .query::<(
            Entity,
            &Vessel,
            &Position,
            &Rotation,
            &Transform,
            &LinearVelocity,
            &AngularVelocity,
        )>()
        .iter(world)
        .map(|(e, v, p, r, t, linear, angular)| {
            (e, v.body.clone(), p.0, r.0, *t, linear.0, angular.0)
        })
        .collect();
    for (entity, body, position, rotation, display, velocity, angular) in bodies {
        if !world
            .get::<aether_view::BodyVisual>(entity)
            .is_some_and(|v| v.body.blueprint() == body.blueprint())
        {
            return Err(format!("body {entity:?}: visual blueprint differs"));
        }
        for part in body.parts() {
            let local = part.center();
            let shown = display.transform_point(local);
            let physical = position + rotation * local;
            // Interpolation intentionally trails physics by at most one fixed step.
            if shown.distance(physical)
                > (velocity.length() + angular.length() * local.length()) / 60.0 * 1.1 + 0.005
            {
                return Err(format!(
                    "body {entity:?} part {:?}: interpolated pose differs by {}m",
                    part.id,
                    shown.distance(physical)
                ));
            }
            if !part.kind.is_sail() {
                let visual = world
                    .query::<(&aether_view::PartVisual, &GlobalTransform)>()
                    .iter(world)
                    .find(|(p, _)| p.owner == entity && p.id == part.id);
                if !visual.is_some_and(|(_, t)| t.translation().distance(shown) < 0.002) {
                    return Err(format!(
                        "body {entity:?} part {:?} {:?}: equipment position {:?}, expected {shown:?}",
                        part.id,
                        part.kind,
                        visual.map(|(_, t)| t.translation())
                    ));
                }
            }
        }
        let Some((column, _)) = body.grid().iter().next() else {
            return Err(format!("body {entity:?}: empty grid"));
        };
        let expected = body
            .grid()
            .iter()
            .filter(|(c, _)| c.0 == column.0 && c.2 == column.2)
            .max_by_key(|(c, _)| c.1)
            .unwrap()
            .0;
        let local = expected.center() + Vec3::Y * 2.0;
        let inverse = display.to_matrix().inverse();
        let origin = inverse.transform_point3(display.transform_point(local));
        if !aether_core::picking::raycast(body.grid(), origin, -Vec3::Y, 5.0)
            .is_some_and(|hit| hit.cell == expected && (hit.distance - 1.75).abs() < 0.002)
        {
            return Err(format!(
                "body {entity:?}: picking failed for {expected:?} at {origin:?}"
            ));
        }
    }
    Ok(())
}
pub fn verify_presentation(world: &mut World) {
    if !world.resource::<Diagnostics>().scripted
        || world.resource::<aether_view::MeshMetrics>().pending > 0
    {
        return;
    }
    let result = presentation_matches_physics(world);
    let mut diagnostics = world.resource_mut::<Diagnostics>();
    diagnostics.presentation_valid = result.is_ok();
    diagnostics.presentation_failure = result.err();
}
pub fn update(world: &mut World) {
    world.resource_scope(|world, mut d: Mut<Diagnostics>| {
        let dt = world.resource::<Time<Real>>().delta_secs();
        d.elapsed += dt; d.wait += dt;
        if d.benchmark && !d.benchmark_started && *world.resource::<State<Phase>>().get()==Phase::Menu {
            let mut saved=session::starter();let template=saved.vessels[0].clone();
            let mut parts=Vec::new();
            for x in 0..10 {for z in 0..10 {parts.push(aether_core::Part{id:aether_core::PartId(parts.len()as u64+1),kind:aether_core::PartKind::Lift,cell:Cell(-45+x*10,0,-45+z*10),quarter_turn:0});}}
            for x in 0..10 {parts.push(aether_core::Part{id:aether_core::PartId(parts.len()as u64+1),kind:aether_core::PartKind::Tank,cell:Cell(-40+x*8,0,35),quarter_turn:0});}
            for (kind,cell) in [(aether_core::PartKind::Helm,Cell(0,0,44)),(aether_core::PartKind::Sail,Cell(0,0,0)),(aether_core::PartKind::Harpoon,Cell(0,0,-49))]{parts.push(aether_core::Part{id:aether_core::PartId(parts.len()as u64+1),kind,cell,quarter_turn:0});}
            saved.vessels[0].blueprint=aether_core::Blueprint{circuit_design:None,name:"Banc 10 000 cellules".into(),cells:(-50..50).flat_map(|x|(-50..50).map(move|z|(Cell(x,0,z),Block::Wood))).collect(),parts};
            saved.vessels[0].fuel=1800.0;saved.vessels[0].docked=false;
            for index in 0..31 {let mut v=template.clone();v.id=aether_core::BodyId(index+2);v.docked=false;v.position=Vec3::new((index%6)as f32*22.0-55.0,45.0,(index/6)as f32*22.0-44.0);v.target_altitude=Some(45.0);saved.vessels.push(v);}
            session::replace(world,saved);world.resource_mut::<NextState<Phase>>().set(Phase::Playing);
            world.resource_mut::<crate::camera::CameraRig>().distance=65.0;
            d.benchmark_started=true;d.elapsed=0.0;d.frames.clear();d.ticks.clear();
        }
        if d.benchmark_started&&d.elapsed>33.0&&d.stage==0 {capture(world,&d.output,"benchmark");d.stage=1;}
        if d.benchmark_started && d.elapsed>35.0&&!d.finished {
            check(&mut d,world.query::<&Vessel>().iter(world).count()==32,"scène 32 corps");
            check(&mut d,world.get::<Vessel>(active(world)).is_some_and(|v|v.body.grid().len()==10_000),"construction 10k");
            d.finished=true;
        }
        if d.elapsed > 5.0 {
            for diagnostic in world.resource::<bevy::diagnostic::DiagnosticsStore>().iter() {
                let name=diagnostic.path().to_string();
                if !(name.ends_with("elapsed_gpu") || name.ends_with("elapsed_cpu")) { continue; }
                if let Some(measurement)=diagnostic.measurement() {
                    if d.render_last.get(&name)==Some(&measurement.time) { continue; }
                    d.render_last.insert(name.clone(),measurement.time);
                    let values=d.render_times.entry(name).or_default();
                    values.push_back(measurement.value);
                    if values.len()>3600 {values.pop_front();}
                }
            }
            d.frames.push_back(dt as f64 * 1000.0);
            if d.frames.len() > 3600 { d.frames.pop_front(); }
        }
        if world.resource::<ButtonInput<KeyCode>>().just_pressed(KeyCode::F3) { d.enabled = !d.enabled; }
        if d.scripted && !d.finished && d.failure.is_none() && d.wait > 0.4 {
            if d.elapsed > 120.0 { d.failure = Some(format!("timeout stage {}", d.stage)); }
            let phase = *world.resource::<State<Phase>>().get();
            let busy = world.resource::<Storage>().pending || world.resource::<aether_view::MeshMetrics>().pending > 0;
            if !busy && matches!(phase, Phase::Menu | Phase::Playing | Phase::Editing | Phase::Paused) {
                let base_cells = aether_core::fixtures::explorer().grid().len();
                let mut advance = true;
                let pose_ok=d.presentation_valid;
                match d.stage {
                    0 => action(world, Action::NewGame),
                    1 => { check(&mut d, phase == Phase::Playing, "nouvelle partie"); action(world, Action::Edit); }
                    2 => {
                        check(&mut d, phase == Phase::Editing, "atelier au quai");
                        editor::execute(world, Edit::Set(Cell(5,0,2), Block::Wood));
                    }
                    3 => { check(&mut d, world.get::<Vessel>(active(world)).unwrap().body.grid().len() == base_cells+1, "ajout transactionnel"); action(world, Action::Undo); }
                    4 => { check(&mut d, world.get::<Vessel>(active(world)).unwrap().body.grid().len() == base_cells, "annulation"); action(world, Action::Redo); }
                    5 => { check(&mut d, world.get::<Vessel>(active(world)).unwrap().body.grid().len() == base_cells+1, "rétablissement"); action(world, Action::Save); }
                    6 => { check(&mut d, world.resource::<GameSession>().notice.contains("enregistrée"), "sauvegarde confirmée"); action(world, Action::NewGame); }
                    7 => { check(&mut d, world.get::<Vessel>(active(world)).unwrap().body.grid().len() == base_cells, "nouvelle session nettoyée"); action(world, Action::Load); }
                    8 => { check(&mut d, world.get::<Vessel>(active(world)).unwrap().body.grid().len() == base_cells+1, "reprise de la construction");let detail=format!("poses, équipements et picking après reprise{}",d.presentation_failure.as_ref().map(|s|format!(": {s}")).unwrap_or_default());check(&mut d,pose_ok,&detail); action(world, Action::Walk); }
                    9 => { check(&mut d, world.resource::<GameSession>().walker.is_some(), "marche et incarnation"); action(world, Action::Save); }
                    10 => action(world, Action::Load),
                    11 => { check(&mut d, world.resource::<GameSession>().walker.is_some(), "reprise du personnage"); action(world, Action::Walk); }
                    12 => action(world, Action::Dock),
                    13 => {
                        let e = active(world);
                        check(&mut d, !world.get::<Vessel>(e).unwrap().docked, "départ");
                        let p = Vec3::new(15.0,24.0,-64.0);
                        world.entity_mut(e).insert((aether_sim::teleport_pose(p,Quat::IDENTITY), LinearVelocity::ZERO, AngularVelocity::ZERO));
                        world.get_mut::<Vessel>(e).unwrap().target_altitude = p.y;
                    }
                    14 => action(world, Action::Tether),
                    15 => { check(&mut d, world.query::<&aether_sim::Tether>().iter(world).count()==1, "harpon et joint unique"); action(world, Action::Save); }
                    16 => action(world, Action::Load),
                    17 => { check(&mut d, world.query::<&aether_sim::Tether>().iter(world).count()==1, "reconstruction des attaches");check(&mut d,pose_ok,"point local après reprise attachée"); action(world, Action::Tether); }
                    18 => { check(&mut d, world.query::<&aether_sim::Tether>().iter(world).count()==0, "libération du câble"); action(world, Action::Recover); }
                    19 => { check(&mut d, world.get::<Vessel>(active(world)).unwrap().docked, "secours au quai"); action(world, Action::Edit); }
                    20 => editor::execute(world, Edit::Set(Cell(10,0,0), Block::Wood)),
                    21 => action(world, Action::Split),
                    22 => { check(&mut d, world.query::<&Vessel>().iter(world).count()==2, "scission exacte");check(&mut d,pose_ok,"poses et picking après scission"); action(world, Action::Undo); }
                    23 => { check(&mut d, world.query::<&Vessel>().iter(world).count()==1, "annulation de scission"); action(world, Action::Redo); }
                    24 => { check(&mut d, world.query::<&Vessel>().iter(world).count()==2, "rétablissement de scission"); action(world, Action::NewGame); }
                    25 => {
                        let mut environment: Vec<_> = world.query_filtered::<Entity, With<aether_view::environment::EnvironmentVisual>>().iter(world).collect();
                        environment.sort();
                        if d.environment_entities.is_empty() {
                            d.environment_entities = environment;
                        } else {
                            let preserved = environment == d.environment_entities;
                            check(&mut d, preserved, "décor conservé entre les reprises");
                        }
                        let count = world.entity_count();
                        if let Some(&baseline) = d.entity_samples.first() {
                            check(&mut d, count <= baseline + 8, "nombre total d'entités stable");
                        }
                        d.entity_samples.push(count);
                        let assets=[world.resource::<Assets<Mesh>>().len(),world.resource::<Assets<StandardMaterial>>().len(),world.resource::<aether_view::MeshMetrics>().pending];
                        if let Some(baseline)=d.asset_samples.first().copied() {
                            check(&mut d,assets[0]<=baseline[0]+8&&assets[1]<=baseline[1]+2&&assets[2]==0,"maillages, matériaux et file stables");
                        }
                        d.asset_samples.push(assets);
                        check(&mut d, world.query::<&Vessel>().iter(world).count()==1, "nettoyage des fragments");
                        check(&mut d, world.query_filtered::<Entity, With<crate::camera::MainCamera>>().iter(world).count()==1, "caméra unique");
                        check(&mut d, world.query_filtered::<Entity, With<crate::interface::UiRoot>>().iter(world).count()==1, "interface unique");
                        d.cycles += 1;
                        if d.cycles < 20 { action(world, Action::NewGame); advance=false; }
                    }
                    26 => { action(world, Action::Edit); }
                    27 => {
                        for i in 0..100 { editor::execute(world, Edit::Set(Cell(5+i%10,0,2+i/10), Block::Wood)); }
                    }
                    28 => {
                        for _ in 0..100 { editor::history(world,false); }
                        check(&mut d, world.get::<Vessel>(active(world)).unwrap().body.grid().len()==base_cells, "100 éditions puis annulations");
                    }
                    29 => {
                        capture(world, &d.output, "atelier");
                    }
                    30 => {
                        let mut saved = session::snapshot(world).unwrap();
                        saved.vessels[0].position = aether_sim::ISLANDS[1].dock;
                        saved.vessels[0].docked = false;
                        session::replace(world, saved);
                    }
                    31 => action(world, Action::Dock),
                    32 => {
                        check(&mut d, world.resource::<GameSession>().progress==4, "arrivée et checkpoint");
                        action(world, Action::Recover);
                    }
                    33 => { capture(world, &d.output, "arrivee"); }
                    34 => { d.finished = true; }
                    _ => {}
                }
                if advance { d.stage += 1; }
                d.wait = 0.0;
            }
        }
        let phase = *world.resource::<State<Phase>>().get();
        if d.scripted && phase == Phase::Error { d.failure = Some("chargement des assets".into()); }
        if d.failure.is_some() { d.finished = true; }
        // Cheap, read-only per-frame probe: the full report below is deliberately
        // throttled and must not add 200 ms to command/mesh latency measurements.
        #[cfg(target_arch="wasm32")]
        if d.probe {
            let cells = world.resource::<GameSession>().active
                .and_then(|e| world.get::<Vessel>(e)).map_or(0, |v| v.body.grid().len());
            publish_edit_probe(cells as u32, world.resource::<aether_view::MeshMetrics>().pending as u32);
        }
        if d.elapsed - d.published < 0.2 && !d.finished { return; }
        d.published = d.elapsed;
        let scale = world.query::<&Window>().iter(world).next().map_or(1.0, |w| w.scale_factor());
        let window_focused = world.query::<&Window>().iter(world).next().is_some_and(|w| w.focused);
        let ui_targets: Vec<_> = if d.probe {
            world.query::<(Entity, &crate::interface::UiAction, &UiGlobalTransform, &ComputedNode, &InheritedVisibility, &crate::interface::ButtonStyle)>().iter(world).filter(|(_,_,_,_,visible,_)|visible.get()).map(|(entity,action,t,node,_,style)| {
                let p=t.affine().translation/scale; let size=node.size()/scale;
                json!({"entity":entity.to_bits(),"action":format!("{:?}",action.0),"x":p.x,"y":p.y,"width":size.x,"height":size.y,"selected":style.0})
            }).collect()
        } else { vec![] };
        let edit_world = world.resource::<GameSession>().active.and_then(|e|world.get::<Transform>(e))
            .map(|t| t.transform_point(Vec3::new(0.0,0.25,2.0)));
        let edit_point = edit_world.and_then(|point| world.query_filtered::<(&Camera,&GlobalTransform),With<crate::camera::MainCamera>>()
            .iter(world).next().and_then(|(camera,t)| camera.world_to_viewport(t,point).ok())).map(|p|[p.x,p.y]);
        let avatar_fallbacks=world.query_filtered::<Entity,With<aether_view::avatar::AvatarFallback>>().iter(world).count();
        let active_sail_owner = world.resource::<GameSession>().active;
        let sail_motion: Vec<_> = world.query::<(&aether_view::sail::SailSurface, &bevy::mesh::morph::MorphWeights)>()
            .iter(world).filter_map(|(surface, weights)| {
                let part = world.get::<aether_view::PartVisual>(surface.0)?;
                if Some(part.owner) != active_sail_owner { return None; }
                let wind = world.get::<aether_view::sail::SailWind>(surface.0)?;
                Some(json!({"weights": weights.weights(), "normal_wind": wind.normal_speed, "speed": wind.speed, "seconds": wind.seconds}))
            }).collect();
        let residents: Vec<_> = world.query::<(&aether_sim::fauna::Resident, &Position)>()
            .iter(world).map(|(r,p)|json!({"id":r.0.id,"species":r.0.species,"position":p.0.to_array()})).collect();
        let couriers: Vec<_> = world.query::<(&aether_sim::traffic::Courier, &Position)>()
            .iter(world).map(|(r,p)|json!({"id":r.0.id,"position":p.0.to_array()})).collect();
        let game = world.resource::<GameSession>();
        let contacts=world.resource::<ContactGraph>();
        let contact_pairs=contacts.iter_active_touching().count();
        let sleeping_pairs=contacts.iter_sleeping_touching().count();
        let contact_manifolds=contacts.iter_active_touching().map(|p|p.manifolds.len()).sum::<usize>();
        let triangles=world.resource::<Assets<Mesh>>().iter().map(|(_,m)|m.indices().map_or(m.count_vertices()/3,|i|i.len()/3)).sum::<usize>();
        let vessel = game.active.and_then(|e| world.get::<Vessel>(e));
        let navigation=game.active.and_then(|e|{
            let p=world.get::<Position>(e)?.0;let r=world.get::<Rotation>(e)?.0;
            let velocity=world.get::<LinearVelocity>(e)?.0;let angular=world.get::<AngularVelocity>(e)?.0;
            let input=world.get::<aether_sim::PilotIntent>(e)?;
            Some(json!({"position":p.to_array(),"yaw":r.to_euler(EulerRot::YXZ).0,"velocity":velocity.to_array(),"angular_y":angular.y,"target_altitude":world.get::<Vessel>(e)?.target_altitude,
                "intent":{"throttle":input.throttle,"turn":input.turn,"climb":input.climb,"brake":input.brake}}))
        });
        let mesh = world.resource::<aether_view::MeshMetrics>();
        #[cfg(target_arch="wasm32")]
        let wasm_memory_bytes=Some(core::arch::wasm32::memory_size::<0>()*65536);
        #[cfg(not(target_arch="wasm32"))]
        let wasm_memory_bytes:Option<usize>=None;
        let mut snapshot = json!({
            "phase":format!("{phase:?}"), "tick":world.resource::<SimClock>().tick,
            "progress":game.progress,"walker":game.walker.is_some(),"notice":game.notice,
            "cells":vessel.map_or(0,|v|v.body.grid().len()),"fuel":vessel.map_or(0.0,|v|v.fuel),
            "docked":vessel.is_some_and(|v|v.docked),"pending_storage":world.resource::<Storage>().pending,
            "mesh_pending":mesh.pending,"mesh_p95_ms":mesh.p95(),"mesh_max_ms":mesh.max_ms,
            "mesh_apply_cpu_ms_p50_p95_p99":percentiles(&mesh.apply_ms),"mesh_queue_to_apply_ms_p50_p95_p99":percentiles(&mesh.queue_to_apply_ms),
            "frame_ms_p50_p95_p99":percentiles(&d.frames),"tick_ms_p50_p95_p99":percentiles(&d.ticks),
            "entities":world.entity_count(),"allocated_entity_indices":world.entities().len(),"elapsed_seconds":d.elapsed,"qa_stage":d.stage,
            "contact_pairs_active":contact_pairs,"contact_pairs_sleeping":sleeping_pairs,"contact_manifolds_active":contact_manifolds,
            "triangles_in_mesh_assets":triangles,"tutorial_flags":world.resource::<crate::persistence::Preferences>().tutorial,
            "bindings":world.resource::<crate::persistence::Preferences>().bindings,
            "binding_message":world.resource::<crate::bindings::BindingCapture>().message,
            "avatar_fallbacks":avatar_fallbacks,
            "benchmark":d.benchmark,"gpu_timestamps_measured":d.render_times.keys().any(|k|k.ends_with("elapsed_gpu")),
            "render_pass_ms_p50_p95_p99":d.render_times.iter().map(|(name,values)|(name.clone(),percentiles(values))).collect::<BTreeMap<_,_>>(),
            "navigation":navigation,
            "wasm_linear_memory_bytes":wasm_memory_bytes,
            "mesh_assets":world.resource::<Assets<Mesh>>().len(),"material_assets":world.resource::<Assets<StandardMaterial>>().len(),
            "entity_samples_20_cycles":d.entity_samples,
            "ui_targets":ui_targets,"edit_point":edit_point,
            "qa_finished":d.finished,"qa_failure":d.failure,"checks":d.checks,
        });
        snapshot["binding_selected"] = json!(world.resource::<crate::bindings::BindingCapture>().selected);
        snapshot["binding_active"] = json!(world.resource::<crate::bindings::BindingCapture>().active);
        if d.probe {
            snapshot["travel_destinations"] = json!(crate::travel::destinations().iter().map(|d|json!({"id":d.id,"label":d.label,"island":d.island,"position":d.position.to_array()})).collect::<Vec<_>>());
        }
        snapshot["decoration_cpu_ms_p50_p95_p99"] = json!(percentiles(&mesh.decoration_ms));
        if let Some(loading) = world.get_resource::<crate::app::LoadingStatus>() {
            let server = world.resource::<AssetServer>();
            let pending: Vec<_> = loading.assets.iter()
                .filter(|(_,handle)| !server.is_loaded_with_dependencies(handle.id()))
                .map(|(path,handle)| json!({"path":path,"states":format!("{:?}",server.get_load_states(handle.id()))}))
                .collect();
            snapshot["loading"] = json!({"elapsed":loading.elapsed,"error":loading.error,"pending":pending});
        }
        snapshot["window_focused"] = json!(window_focused);
        snapshot["physical_keys"] = json!(world.resource::<ButtonInput<KeyCode>>().get_pressed().map(|k|format!("{k:?}")).collect::<Vec<_>>());
        snapshot["sail_motion"] = json!(sail_motion);
        snapshot["expedition"] = json!(game.expedition);
        snapshot["checkpoint"] = json!(game.checkpoint);
        snapshot["residents"] = json!(residents);
        snapshot["couriers"] = json!(couriers);
        snapshot["mass_kg"] = json!(vessel.map_or(0.0,|v|v.properties.mass));
        snapshot["keyboard_ui"]=json!(world.resource::<crate::interface::KeyboardUi>().active);
        snapshot["keyboard_action"]=json!(world.resource::<crate::interface::KeyboardUi>().focused.and_then(|e|world.get::<crate::interface::UiAction>(e)).map(|a|format!("{:?}",a.0)));
        snapshot["asset_samples_20_cycles"]=json!(d.asset_samples);
        snapshot["physics_phase_ms_p50_p95_p99"]=json!(d.physics_phases.iter().map(|(name,values)|(name.clone(),percentiles(values))).collect::<BTreeMap<_,_>>());
        let camera=world.resource::<crate::camera::CameraRig>();
        snapshot["camera"]=json!({"yaw":camera.yaw,"pitch":camera.pitch,"distance":camera.distance});
        if d.enabled {
            let message = format!("DEV  F3\n{:?} | tick {}\nframe p95 {:.2} ms | tick p95 {:.2} ms\nmesh p95 {:.2} ms | file {}\nentités {} | triangles {}\ncontacts {} actifs / {} endormis | manifolds {}",phase,world.resource::<SimClock>().tick,percentiles(&d.frames)[1],percentiles(&d.ticks)[1],mesh.p95(),mesh.pending,world.entity_count(),triangles,contact_pairs,sleeping_pairs,contact_manifolds);
            for mut text in world.query_filtered::<&mut Text, With<DebugText>>().iter_mut(world) { text.0 = message.clone(); }
        }
        for mut visibility in world.query_filtered::<&mut Visibility, With<DebugText>>().iter_mut(world) { *visibility = if d.enabled { Visibility::Visible } else { Visibility::Hidden }; }
        #[cfg(target_arch="wasm32")]
        if d.probe { publish_probe(&snapshot.to_string()); }
        #[cfg(not(target_arch="wasm32"))]
        if d.finished && !d.output.is_empty() {
            let path = std::path::Path::new(&d.output);
            if let Some(parent)=path.parent() { let _ = std::fs::create_dir_all(parent); }
            if let Err(error)=std::fs::write(path,serde_json::to_vec_pretty(&snapshot).unwrap()) { error!("QA report: {error}"); }
            world.write_message(if d.failure.is_some() { AppExit::error() } else { AppExit::Success });
        }
    });
}
fn capture(world: &mut World, output: &str, name: &str) {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(parent) = std::path::Path::new(output).parent() {
        let path = parent.join(format!("qa-{name}.png"));
        world
            .commands()
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(path));
    }
    #[cfg(target_arch = "wasm32")]
    let _ = (world, output, name);
}
pub fn setup(mut commands: Commands) {
    commands.spawn((
        DebugText,
        aether_view::widgets::label("", 12.0, Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: px(320),
            bottom: px(75),
            padding: UiRect::all(px(12)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        GlobalZIndex(100),
        Visibility::Hidden,
    ));
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function publish_probe(value){window.aetherProbe=JSON.parse(value);} export function publish_edit_probe(cells,pending){window.aetherEditProbe={cells,mesh_pending:pending};}"
)]
extern "C" {
    fn publish_probe(value: &str);
    fn publish_edit_probe(cells: u32, pending: u32);
}
