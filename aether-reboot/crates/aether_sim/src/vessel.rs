use aether_core::{Body, BodyId, CELL_SIZE, save::VesselSave};
use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct Vessel {
    pub id: BodyId,
    pub body: Body,
    pub properties: aether_core::MassProperties,
    pub payload: f32,
    pub fuel: f32,
    pub trim: f32,
    pub docked: bool,
    pub target_altitude: f32,
}
#[derive(Component, Clone, Copy, Default, Debug)]
pub struct PilotIntent {
    pub throttle: f32,
    pub turn: f32,
    pub climb: f32,
    pub trim: f32,
    pub brake: bool,
}
#[derive(Component, Default, Clone, Copy)]
pub struct FlightTelemetry {
    pub lift_ratio: f32,
    pub consumption: f32,
    pub wind: Vec3,
    pub current_weight: f32,
    pub motor: f32,
    pub leakage: f32,
    pub circuit_fault: bool,
}
#[derive(Component)]
pub struct SessionEntity;
#[derive(Component)]
pub struct HullCollider;
#[derive(Component)]
pub struct EquipmentCollider;
/// Teleporting is an explicit discontinuity: reset presentation history as well as physics.
/// In particular, changing a dynamic body to static must not retain its old eased pose.
pub fn teleport_pose(position: Vec3, rotation: Quat) -> impl Bundle {
    (
        Position(position),
        Rotation(rotation),
        Transform::from_translation(position).with_rotation(rotation),
        bevy_transform_interpolation::TranslationEasingState::default(),
        bevy_transform_interpolation::RotationEasingState::default(),
    )
}
pub fn spawn_equipment(commands: &mut Commands, owner: Entity, body: &Body) {
    let rails: Vec<_> = aether_core::naval::fittings(body)
        .into_iter()
        .filter(|f| f.kind == aether_core::naval::Kind::Rail)
        .map(|f| {
            (
                f.position + Vec3::Y * 0.3,
                f.rotation,
                Collider::cuboid(0.50, 0.60, 0.064),
            )
        })
        .collect();
    if !rails.is_empty() {
        commands.spawn((
            EquipmentCollider,
            Collider::compound(rails),
            Transform::default(),
            ColliderDensity(0.0),
            Friction::new(0.7),
            ChildOf(owner),
        ));
    }
    for part in body.parts() {
        let (center, size) = if part.kind.is_sail() {
            let scale = if part.kind == aether_core::PartKind::GrandSail {
                2.5
            } else {
                1.0
            };
            (
                part.cell.center() + Vec3::Y * 1.65 * scale,
                Vec3::new(0.13, 3.3, 0.13) * scale,
            )
        } else {
            (part.center(), part.kind.size())
        };
        commands.spawn((
            EquipmentCollider,
            Collider::cuboid(size.x, size.y, size.z),
            Transform::from_translation(center).with_rotation(Quat::from_rotation_y(
                part.quarter_turn as f32 * std::f32::consts::FRAC_PI_2,
            )),
            ColliderDensity(0.0),
            Friction::new(0.7),
            Restitution::new(0.0),
            ChildOf(owner),
        ));
    }
}

/// Parry voxel coordinates address cell corners. Attach with `collider_offset()`.
pub fn collider(body: &Body) -> Collider {
    let coordinates: Vec<_> = body.grid().iter().map(|(cell, _)| cell.ivec()).collect();
    Collider::voxels(Vec3::splat(CELL_SIZE), &coordinates)
}
pub fn collider_offset() -> Transform {
    Transform::from_translation(Vec3::splat(-CELL_SIZE * 0.5))
}
pub fn spawn_vessel(commands: &mut Commands, save: VesselSave) -> Entity {
    let body =
        Body::from_blueprint(save.blueprint).expect("session must be validated before spawning");
    let circuit = save.circuit.map(|state| {
        crate::aether::AetherCircuit::restore(&body, state)
            .expect("session circuit must be validated before spawning")
    });
    let mass = body.mass_properties();
    let shape = collider(&body);
    let equipment = body.clone();
    let vessel = Vessel {
        id: save.id,
        body,
        properties: mass,
        payload: 0.0,
        fuel: save.fuel,
        trim: save.trim,
        docked: save.docked,
        target_altitude: save.target_altitude.unwrap_or(save.position.y).clamp(
            aether_core::tuning::ALTITUDE_MIN_METERS,
            aether_core::tuning::ALTITUDE_MAX_METERS,
        ),
    };
    let entity = commands
        .spawn((
            Name::new("Vaisseau"),
            SessionEntity,
            SweptCcd::LINEAR,
            TransformInterpolation,
            vessel,
            PilotIntent::default(),
            FlightTelemetry::default(),
            Transform::from_translation(save.position).with_rotation(save.rotation),
            if save.docked {
                RigidBody::Static
            } else {
                RigidBody::Dynamic
            },
            (
                Mass(mass.mass),
                CenterOfMass(mass.center),
                AngularInertia::from_mat3_unchecked(mass.inertia),
                ColliderDensity(0.0),
                LinearVelocity(save.velocity),
                AngularVelocity(save.angular_velocity),
                ConstantForce::default(),
                ConstantTorque::default(),
                Friction::new(0.7),
                Restitution::new(0.0),
            ),
        ))
        .id();
    commands.spawn((
        HullCollider,
        shape,
        collider_offset(),
        ColliderDensity(0.0),
        Friction::new(0.7),
        Restitution::new(0.0),
        ChildOf(entity),
    ));
    spawn_equipment(commands, entity, &equipment);
    if let Some(circuit) = circuit {
        commands.entity(entity).insert(circuit);
    }
    entity
}
pub fn capture(
    vessel: &Vessel,
    position: Vec3,
    rotation: Quat,
    velocity: Vec3,
    angular_velocity: Vec3,
) -> VesselSave {
    VesselSave {
        circuit: None,
        id: vessel.id,
        blueprint: vessel.body.blueprint(),
        position,
        rotation,
        velocity,
        angular_velocity,
        fuel: vessel.fuel,
        trim: vessel.trim,
        target_altitude: Some(vessel.target_altitude),
        docked: vessel.docked,
    }
}
/// Rebuild collision and derived mass together at a simulation boundary.
pub fn replace_geometry(commands: &mut Commands, entity: Entity, vessel: &Vessel) {
    let mass = vessel.body.mass_properties().with_payload(vessel.payload);
    commands.entity(entity).insert((
        Mass(mass.mass),
        CenterOfMass(mass.center),
        AngularInertia::from_mat3_unchecked(mass.inertia),
    ));
}
