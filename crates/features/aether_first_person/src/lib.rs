use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

pub struct FirstPersonPlugin;

#[derive(Component)]
pub struct FirstPersonPlayer {
    pub speed: f32,
    pub mouse_sensitivity: f32,
    pitch: f32,
    yaw: f32,
}

impl Default for FirstPersonPlayer {
    fn default() -> Self {
        Self {
            speed: 8.0,
            mouse_sensitivity: 0.002,
            pitch: -0.18,
            yaw: 0.0,
        }
    }
}

impl Plugin for FirstPersonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (move_player, look_player));
    }
}

fn spawn_player(mut commands: Commands) {
    let controller = FirstPersonPlayer::default();
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 8.0).with_rotation(Quat::from_euler(
            EulerRot::YXZ,
            controller.yaw,
            controller.pitch,
            0.0,
        )),
        controller,
    ));
}

fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&FirstPersonPlayer, &mut Transform)>,
) {
    for (player, mut transform) in &mut players {
        let mut input = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyS) {
            input.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyW) {
            input.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            input.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            input.x += 1.0;
        }
        if keyboard.pressed(KeyCode::Space) {
            input.y += 1.0;
        }
        if keyboard.pressed(KeyCode::ShiftLeft) {
            input.y -= 1.0;
        }

        if input != Vec3::ZERO {
            let forward = transform.forward().as_vec3();
            let right = transform.right().as_vec3();
            let horizontal = right * input.x + forward * input.z;
            transform.translation += (horizontal + Vec3::Y * input.y).normalize_or_zero()
                * player.speed
                * time.delta_secs();
            transform.translation.y = transform.translation.y.max(0.25);
        }
    }
}

fn look_player(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut players: Query<(&mut FirstPersonPlayer, &mut Transform)>,
) {
    let delta = mouse_motion.read().map(|event| event.delta).sum::<Vec2>();
    if delta == Vec2::ZERO || !mouse_buttons.pressed(MouseButton::Left) {
        return;
    }

    for (mut player, mut transform) in &mut players {
        player.yaw -= delta.x * player.mouse_sensitivity;
        player.pitch = (player.pitch - delta.y * player.mouse_sensitivity).clamp(-1.54, 1.54);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
    }
}
