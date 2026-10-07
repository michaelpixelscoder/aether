use bevy::prelude::*;
#[derive(Resource)]
pub struct TrafficArt(pub [Handle<bevy::world_serialization::WorldAsset>; 2]);
#[derive(Component)]
pub struct CourierRotor(pub Quat);
pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(TrafficArt(["trader", "trader-lod"].map(|key| {
        assets.load(GltfAssetLabel::Scene(0).from_asset(format!("fauna/{key}.glb")))
    })));
}
pub fn spawn(
    commands: &mut Commands,
    art: &TrafficArt,
    owner: Entity,
    courier: aether_core::traffic::Courier,
) {
    if let Some(body) = courier.sailing_body() {
        commands.entity(owner).insert(crate::BodyVisual::new(body));
        return;
    }
    for n in 0..2 {
        commands
            .spawn((
                WorldAssetRoot(art.0[n].clone()),
                Transform::IDENTITY,
                ChildOf(owner),
                crate::world::IslandLod {
                    near: n == 0,
                    distance: 650.0,
                },
            ))
            .observe(crate::world::bind)
            .observe(bind);
    }
}
fn bind(
    event: On<bevy::world_serialization::WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    names: Query<(&Name, &Transform)>,
) {
    for e in children.iter_descendants(event.entity) {
        if let Ok((name, t)) = names.get(e)
            && name.as_str().starts_with("MerchantPropeller")
        {
            commands.entity(e).insert(CourierRotor(t.rotation));
        }
    }
}
pub fn animate(
    view: Res<crate::weather::WeatherView>,
    mut rotors: Query<(&CourierRotor, &mut Transform)>,
) {
    if view.reduced_motion {
        return;
    }
    for (r, mut t) in &mut rotors {
        t.rotation = r.0 * Quat::from_rotation_z(view.seconds * 19.0);
    }
}
