use crate::SessionEntity;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::builtins::{
    TnuaBuiltinJump, TnuaBuiltinJumpConfig, TnuaBuiltinWalk, TnuaBuiltinWalkConfig,
};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::prelude::*;

#[derive(TnuaScheme)]
#[scheme(basis=TnuaBuiltinWalk)]
pub enum Walking {
    Jump(TnuaBuiltinJump),
}
pub type WalkerController = TnuaController<Walking>;
#[derive(Component, Default)]
pub struct WalkerIntent {
    pub direction: Vec3,
    pub jump: bool,
}
#[derive(Component)]
pub struct Walker;
/// A bounded standing correction, fed as a Tnua walk intention. No pose parenting
/// or teleportation: Tnua retains authority over contact, jumping and forces.
#[derive(Component, Default)]
pub struct StandingReference(Option<(Entity, Vec3)>);
/// Tangential velocity of the last support, retained through the ballistic phase.
#[derive(Component, Default)]
pub struct InheritedMotion(Vec3);
impl InheritedMotion {
    /// Resume an airborne actor without immediately braking its saved momentum.
    pub fn from_velocity(velocity: Vec3) -> Self {
        Self(velocity.with_y(0.0))
    }
}
pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TnuaControllerPlugin::<Walking>::new(PhysicsSchedule),
            TnuaAvian3dPlugin::new(PhysicsSchedule),
        ))
        .add_systems(PhysicsSchedule, feed.in_set(TnuaUserControlsSystems))
        .add_observer(own_sensor);
    }
}
// TnuaSensorOf tracks membership but does not cascade despawning in Tnua 0.32.
// These sensors have no Transform: parenting only ties their lifetime to ours.
fn own_sensor(
    event: On<Add, bevy_tnua::TnuaSensorOf>,
    sensors: Query<&bevy_tnua::TnuaSensorOf>,
    walkers: Query<(), With<Walker>>,
    mut commands: Commands,
) {
    if let Ok(sensor) = sensors.get(event.entity)
        && walkers.contains(sensor.0)
    {
        commands.entity(event.entity).insert(ChildOf(sensor.0));
    }
}
fn feed(
    poses: Query<(
        &Position,
        &Rotation,
        Option<&LinearVelocity>,
        Option<&AngularVelocity>,
        Option<&ComputedCenterOfMass>,
    )>,
    mut query: Query<(
        &WalkerIntent,
        &Position,
        &mut StandingReference,
        &mut InheritedMotion,
        &mut TnuaController<Walking>,
    )>,
) {
    for (intent, position, mut reference, mut inherited, mut controller) in &mut query {
        let mut correction = Vec3::ZERO;
        let support = controller.basis_memory.standing_on_entity();
        if let Some(entity) = support {
            inherited.0 = poses
                .get(entity)
                .map_or(Vec3::ZERO, |(p, r, linear, angular, com)| {
                    let center = p.0 + r.0 * com.map_or(Vec3::ZERO, |c| c.0);
                    let motion = linear.map_or(Vec3::ZERO, |v| v.0)
                        + angular
                            .map_or(Vec3::ZERO, |v| v.0)
                            .cross(position.0 - center);
                    motion.with_y(0.0)
                });
        } else {
            correction = inherited.0;
        }
        if intent.direction.length_squared() < 0.001 && !intent.jump {
            if let Some(support) = controller.basis_memory.standing_on_entity() {
                if let Ok((p, r, ..)) = poses.get(support) {
                    match reference.0 {
                        Some((entity, local)) if entity == support => {
                            let error = p.0 + r.0 * local - position.0;
                            if error.length_squared() < 4.0 {
                                correction =
                                    (Vec3::new(error.x, 0.0, error.z) * 8.0).clamp_length_max(1.0);
                            } else {
                                reference.0 = None;
                            }
                        }
                        _ => reference.0 = Some((support, r.0.inverse() * (position.0 - p.0))),
                    }
                }
            } else {
                reference.0 = None;
            }
        } else {
            reference.0 = None;
        }
        controller.initiate_action_feeding();
        controller.basis = TnuaBuiltinWalk {
            desired_motion: intent.direction + correction,
            ..default()
        };
        if intent.jump {
            controller.action(Walking::Jump(Default::default()));
        }
    }
}
pub fn spawn_walker(
    commands: &mut Commands,
    configs: &mut Assets<WalkingConfig>,
    position: Vec3,
) -> Entity {
    commands
        .spawn((
            (SessionEntity, Walker, TransformInterpolation),
            StandingReference::default(),
            InheritedMotion::default(),
            WalkerIntent::default(),
            Transform::from_translation(position),
            Visibility::default(),
            RigidBody::Dynamic,
            Mass(65.0),
            AngularInertia::from_mat3_unchecked(Mat3::from_diagonal(Vec3::new(7.9, 2.7, 7.9))),
            ColliderDensity(0.0),
            Collider::capsule(0.25, 0.6),
            LockedAxes::ROTATION_LOCKED,
            TnuaController::<Walking>::default(),
            TnuaConfig::<Walking>(configs.add(WalkingConfig {
                basis: TnuaBuiltinWalkConfig {
                    float_height: aether_core::tuning::WALKER_FLOAT_HEIGHT,
                    speed: 1.0,
                    max_slope: 0.7,
                    ..default()
                },
                jump: TnuaBuiltinJumpConfig {
                    height: 1.5,
                    ..default()
                },
            })),
        ))
        .id()
}
