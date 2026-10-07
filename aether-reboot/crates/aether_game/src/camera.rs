use crate::{app::Phase, persistence::Preferences, session::GameSession};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};
#[derive(Component)]
pub struct MainCamera;
#[derive(Resource)]
pub struct CameraRig {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub snap: bool,
    pub target: Vec3,
    pub occlusion_distance: f32,
}
impl Default for CameraRig {
    fn default() -> Self {
        Self {
            yaw: 0.45,
            pitch: 0.30,
            distance: 23.0,
            snap: true,
            target: Vec3::new(0.0, 12.0, 0.0),
            occlusion_distance: 23.0,
        }
    }
}
pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        bevy::light::VolumetricFog {
            step_count: 24,
            jitter: 0.35,
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            far: 18000.0,
            ..default()
        }),
        aether_view::art::camera_effects(&assets),
        DistanceFog {
            color: Color::srgb(0.42, 0.61, 0.80),
            falloff: FogFalloff::Linear {
                start: 550.0,
                end: 7200.0,
            },
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 16.0).looking_at(Vec3::new(0.0, 13.0, 0.0), Vec3::Y),
    ));
    aether_view::art::spawn_sun(&mut commands);
}
pub fn control(
    phase: Res<State<Phase>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    prefs: Res<Preferences>,
    mut rig: ResMut<CameraRig>,
) {
    if matches!(phase.get(), Phase::Playing | Phase::Editing) {
        if buttons.pressed(MouseButton::Right) {
            rig.yaw -= motion.delta.x * 0.005 * prefs.sensitivity;
            rig.pitch = (rig.pitch
                + motion.delta.y
                    * 0.004
                    * prefs.sensitivity
                    * if prefs.invert_y { -1.0 } else { 1.0 })
            .clamp(0.10, 1.35);
        }
        rig.distance = (rig.distance - scroll.delta.y * 1.7).clamp(5.0, 90.0);
    }
}
pub fn follow(
    time: Res<Time<Real>>,
    phase: Res<State<Phase>>,
    game: Res<GameSession>,
    mut rig: ResMut<CameraRig>,
    prefs: Res<Preferences>,
    targets: Query<&Transform, Without<MainCamera>>,
    mut cameras: Query<&mut Transform, With<MainCamera>>,
    mut mesh_focus: ResMut<aether_view::voxel::MeshFocus>,
    spatial: avian3d::prelude::SpatialQuery,
    solids: Query<&avian3d::prelude::RigidBody, Without<aether_sim::Vessel>>,
) {
    let Some(entity) = game.walker.or(game.active) else {
        return;
    };
    let Ok(target) = targets.get(entity) else {
        return;
    };
    let target_point = target.translation + Vec3::Y * 1.3;
    let blend = if rig.snap || prefs.reduce_motion {
        1.0
    } else {
        1.0 - (-6.0 * time.delta_secs()).exp()
    };
    rig.target = rig.target.lerp(target_point, blend);
    rig.snap = false;
    let yaw = if *phase.get() == Phase::Menu {
        0.55
    } else {
        rig.yaw
    };
    let direction = Vec3::new(
        yaw.sin() * rig.pitch.cos(),
        rig.pitch.sin(),
        yaw.cos() * rig.pitch.cos(),
    );
    use avian3d::prelude::{Collider, RigidBody, ShapeCastConfig, SpatialQueryFilter};
    let hit = spatial.cast_shape_predicate(
        &Collider::sphere(0.35),
        rig.target,
        Quat::IDENTITY,
        Dir3::new(direction).expect("camera direction"),
        &ShapeCastConfig {
            max_distance: rig.distance,
            ignore_origin_penetration: true,
            ..default()
        },
        &SpatialQueryFilter::default(),
        &|e| solids.get(e).is_ok_and(|body| *body == RigidBody::Static),
    );
    let clear_distance = hit.map_or(rig.distance, |h| (h.distance - 0.12).max(0.25));
    // Pull in immediately before a ceiling or cliff; ease the arm back out.
    rig.occlusion_distance = if clear_distance < rig.occlusion_distance || prefs.reduce_motion {
        clear_distance
    } else {
        rig.occlusion_distance + (clear_distance - rig.occlusion_distance) * blend
    };
    for mut camera in &mut cameras {
        camera.translation = rig.target + direction * rig.occlusion_distance;
        camera.look_at(rig.target, Vec3::Y);
        mesh_focus.camera = camera.translation;
        mesh_focus.owner = if *phase.get() == Phase::Editing {
            game.active
        } else {
            None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use avian3d::prelude::*;
    #[test]
    fn orbit_stops_before_a_wall_and_recovers_without_moving_the_player() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::transform::TransformPlugin,
            aether_sim::SimulationPlugin,
        ));
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.insert_resource(State::new(Phase::Playing));
        app.init_resource::<GameSession>()
            .init_resource::<Preferences>()
            .init_resource::<aether_view::voxel::MeshFocus>();
        app.insert_resource(CameraRig {
            yaw: 0.0,
            pitch: 0.1,
            target: Vec3::ZERO,
            ..default()
        });
        let player = app.world_mut().spawn(Transform::IDENTITY).id();
        app.world_mut().resource_mut::<GameSession>().active = Some(player);
        let camera = app
            .world_mut()
            .spawn((MainCamera, Transform::IDENTITY))
            .id();
        let wall = app
            .world_mut()
            .spawn((
                RigidBody::Static,
                Collider::cuboid(8.0, 8.0, 1.0),
                Transform::from_xyz(0.0, 3.0, 8.0),
            ))
            .id();
        app.add_systems(Update, follow);
        app.finish();
        app.cleanup();
        for _ in 0..10 {
            app.update();
        }
        let p = app.world().get::<Transform>(camera).unwrap().translation;
        assert!(p.z > 5.0 && p.z < 7.2, "camera penetrates wall: {p:?}");
        app.world_mut().despawn(wall);
        for _ in 0..120 {
            app.update();
        }
        let clear = app.world().get::<Transform>(camera).unwrap().translation;
        assert!(clear.z > 22.0, "camera did not recover: {clear:?}");
        assert_eq!(
            app.world().get::<Transform>(player).unwrap().translation,
            Vec3::ZERO
        );
    }
}
