//! Per-instance moving machinery; simulation supplies the power and clock.
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct MotorVisual {
    pub seconds: f32,
    pub power: f32,
    pub angle: f32,
    pub reduced_motion: bool,
}
#[derive(Component)]
pub struct Rotor(pub Entity);
pub fn bind(
    event: On<bevy::world_serialization::WorldInstanceReady>,
    mut commands: Commands,
    roots: Query<(), With<MotorVisual>>,
    children: Query<&Children>,
    names: Query<&Name>,
) {
    if roots.get(event.entity).is_err() {
        return;
    }
    for child in children.iter_descendants(event.entity) {
        if names
            .get(child)
            .is_ok_and(|n| n.as_str().starts_with("Propeller rotor"))
        {
            commands.entity(child).insert(Rotor(event.entity));
        }
    }
}
pub fn animate(roots: Query<&MotorVisual>, mut rotors: Query<(&Rotor, &mut Transform)>) {
    for (rotor, mut t) in &mut rotors {
        if let Ok(motor) = roots.get(rotor.0) {
            t.rotation = Quat::from_rotation_z(motor.angle);
        }
    }
}
