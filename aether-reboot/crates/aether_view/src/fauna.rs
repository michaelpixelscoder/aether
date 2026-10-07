use aether_core::fauna::Species;
use bevy::{prelude::*, world_serialization::WorldInstanceReady};
#[derive(Resource)]
pub struct FaunaArt(pub Vec<Handle<bevy::world_serialization::WorldAsset>>);
#[derive(Component)]
pub struct FaunaVisual(pub aether_core::fauna::Resident);
#[derive(Component)]
pub struct Joint {
    base: Quat,
    axis: Vec3,
    sign: f32,
    phase: f32,
    flying: bool,
}
pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(FaunaArt(
        Species::ALL
            .iter()
            .map(|s| {
                assets.load(GltfAssetLabel::Scene(0).from_asset(format!("fauna/{}.glb", s.asset())))
            })
            .collect(),
    ));
}
pub fn bind(
    event: On<WorldInstanceReady>,
    roots: Query<&FaunaVisual>,
    children: Query<&Children>,
    names: Query<(&Name, &Transform)>,
    mut commands: Commands,
) {
    let Ok(visual) = roots.get(event.entity) else {
        return;
    };
    for e in children.iter_descendants(event.entity) {
        if let Ok((name, t)) = names.get(e) {
            let n = name.as_str();
            let flying = n == "WingL" || n == "WingR";
            if flying || n.starts_with("GuardianArm") || n.starts_with("GuardianLeg") {
                let sign = if n.ends_with('L') { 1.0 } else { -1.0 }
                    * if n.starts_with("GuardianLeg") {
                        -1.0
                    } else {
                        1.0
                    };
                commands.entity(e).insert(Joint {
                    base: t.rotation,
                    axis: if flying { Vec3::Z } else { Vec3::X },
                    sign,
                    phase: visual.0.phase,
                    flying,
                });
            }
        }
    }
}
pub fn animate(
    view: Res<crate::weather::WeatherView>,
    mut joints: Query<(&Joint, &mut Transform)>,
) {
    if view.reduced_motion {
        return;
    }
    for (j, mut t) in &mut joints {
        let angle = (view.seconds * if j.flying { 5.5 } else { 2.6 } + j.phase).sin()
            * if j.flying { 0.34 } else { 0.22 }
            * j.sign;
        t.rotation = j.base * Quat::from_axis_angle(j.axis, angle);
    }
}
