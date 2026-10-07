use crate::Palette;
use aether_core::fields::{Current, archipelago_current};
use bevy::prelude::*;

#[derive(Component)]
pub struct EnvironmentVisual;
#[derive(Component)]
pub struct CurrentMote {
    pub phase: f32,
    pub lane: f32,
    pub branch: bool,
}

pub fn island(commands: &mut Commands, art: &crate::art::ArtAssets, center: Vec3, index: usize) {
    commands.spawn((
        EnvironmentVisual,
        WorldAssetRoot(art.scenes[5 + index].clone()),
        Transform::from_translation(center),
    ));
}
pub fn dock(commands: &mut Commands, palette: &Palette, position: Vec3) {
    let deck = aether_core::fixtures::solid([14, 1, 20], aether_core::Block::Wood);
    commands.spawn((
        EnvironmentVisual,
        crate::BodyVisual::new(&deck),
        Transform::from_translation(position - Vec3::new(3.25, 1.5, 4.75)),
        Visibility::default(),
    ));
    for x in [-3.4, 3.4] {
        for z in [-4.7, 4.7] {
            let p = position + Vec3::new(x, 0.0, z);
            block(
                commands,
                palette,
                p,
                Vec3::new(0.25, 3.0, 0.25),
                palette.wood.clone(),
            );
            block(
                commands,
                palette,
                p + Vec3::Y * 1.6,
                Vec3::splat(0.36),
                palette.gold.clone(),
            );
        }
    }
}
pub fn anchor(
    commands: &mut Commands,
    scene: Handle<bevy::world_serialization::WorldAsset>,
    position: Vec3,
) {
    commands.spawn((
        EnvironmentVisual,
        WorldAssetRoot(scene),
        Transform::from_translation(position),
    ));
}
pub fn block(
    commands: &mut Commands,
    palette: &Palette,
    position: Vec3,
    scale: Vec3,
    material: Handle<StandardMaterial>,
) -> Entity {
    commands
        .spawn((
            EnvironmentVisual,
            Mesh3d(palette.cube.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(position).with_scale(scale),
        ))
        .id()
}
pub fn sky(commands: &mut Commands, palette: &Palette) {
    for branch in [false, true] {
        let current = if branch {
            aether_core::fields::refuge_current()
        } else {
            archipelago_current()
        };
        commands.spawn((
            EnvironmentVisual,
            Mesh3d(palette.flow_ribbons[usize::from(branch)].clone()),
            MeshMaterial3d(palette.current.clone()),
            bevy::light::NotShadowCaster,
            bevy::light::NotShadowReceiver,
            Transform::default(),
        ));
        for i in 0..60 {
            let phase = i as f32 / 60.0;
            let lane = (i % 3) as f32 - 1.0;
            commands.spawn((
                EnvironmentVisual,
                CurrentMote {
                    phase,
                    lane,
                    branch,
                },
                Mesh3d(palette.cube.clone()),
                MeshMaterial3d(palette.current.clone()),
                Transform::from_translation(current_point(&current, phase) + Vec3::X * lane * 2.0)
                    .with_scale(Vec3::new(0.025, 0.025, 0.22)),
            ));
        }
    }
}
/// Continuous filaments follow a smooth path inside the authoritative current.
/// Vertices carry a feathered alpha profile, rather than opaque guide segments.
pub fn ribbon(current: &Current) -> Mesh {
    let mut surface = aether_core::mesh::Surface::default();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let point = |t: f32| {
        let p = t.clamp(0.0, 1.0) * (current.points.len() - 1) as f32;
        let i = (p as usize).min(current.points.len() - 2);
        let u = p - i as f32;
        let a = current.points[i.saturating_sub(1)];
        let b = current.points[i];
        let c = current.points[i + 1];
        let d = current.points[(i + 2).min(current.points.len() - 1)];
        (b * 2.0
            + (c - a) * u
            + (a * 2.0 - b * 5.0 + c * 4.0 - d) * u * u
            + (-a + b * 3.0 - c * 3.0 + d) * u * u * u)
            * 0.5
    };
    for lane in [-1.0_f32, 0.0, 1.0] {
        for plane in [Vec3::Y, Vec3::X] {
            let bands = [
                (-0.8, 0.0),
                (-0.15, 0.13),
                (-0.025, 0.65),
                (0.025, 0.65),
                (0.15, 0.13),
                (0.8, 0.0),
            ];
            for step in 0..192 {
                let ta = step as f32 / 192.0;
                let tb = (step + 1) as f32 / 192.0;
                let offset =
                    |t: f32| Vec3::new(lane * 1.4, (t * 14.0 + lane).sin() * 0.32, lane * 0.2);
                let a = point(ta) + offset(ta);
                let b = point(tb) + offset(tb);
                for pair in bands.windows(2) {
                    let base = surface.positions.len() as u32;
                    for (p, width, opacity, t) in [
                        (a, pair[0].0, pair[0].1, ta),
                        (b, pair[0].0, pair[0].1, tb),
                        (b, pair[1].0, pair[1].1, tb),
                        (a, pair[1].0, pair[1].1, ta),
                    ] {
                        surface.positions.push((p + plane * width).to_array());
                        surface.normals.push(Vec3::Y.to_array());
                        surface.uvs.push([t, width]);
                        let fade = (t * 18.0).min((1.0 - t) * 18.0).clamp(0.0, 1.0);
                        colors.push([0.7, 0.85, 1.0, opacity * fade]);
                    }
                    surface
                        .indices
                        .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
                }
            }
        }
    }
    let mut mesh = crate::voxel::to_mesh(surface);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh
}
pub fn current_point(current: &Current, t: f32) -> Vec3 {
    let p = t.rem_euclid(1.0) * (current.points.len() - 1) as f32;
    let i = (p as usize).min(current.points.len() - 2);
    current.points[i].lerp(current.points[i + 1], p - i as f32)
}
pub fn animate_current(time: Res<Time>, mut motes: Query<(&CurrentMote, &mut Transform)>) {
    for (mote, mut transform) in &mut motes {
        let current = if mote.branch {
            aether_core::fields::refuge_current()
        } else {
            archipelago_current()
        };
        let t = mote.phase + time.elapsed_secs() * 0.045;
        let point = current_point(&current, t);
        let next = current_point(&current, t + 0.002);
        transform.translation = point + Vec3::new(mote.lane * 2.0, (t * 12.0).sin() * 0.4, 0.0);
        if point.distance_squared(next) < 100.0 {
            transform.look_to(next - point, Vec3::Y);
        }
    }
}
