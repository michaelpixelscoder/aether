use bevy::{prelude::*, world_serialization::WorldInstanceReady};

#[derive(Resource, Clone)]
pub struct AvatarAssets {
    pub scene: Handle<bevy::world_serialization::WorldAsset>,
    graph: Handle<AnimationGraph>,
    animations: Vec<AnimationNodeIndex>,
}
#[derive(Component)]
pub struct AvatarVisual {
    pub owner: Entity,
    pub pilot: bool,
    pub motion: usize,
}
#[derive(Component)]
pub(crate) struct AvatarPlayer {
    root: Entity,
    motion: usize,
}
#[derive(Component)]
pub struct AvatarFallback;
pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let clips = [36, 72, 40].map(|index| {
        assets.load(GltfAssetLabel::Animation(index).from_asset("characters/Knight.glb"))
    });
    let (graph, animations) = AnimationGraph::from_clips(clips);
    commands.insert_resource(AvatarAssets {
        scene: assets.load(GltfAssetLabel::Scene(0).from_asset("characters/Knight.glb")),
        graph: graphs.add(graph),
        animations,
    });
}
pub fn spawn(
    commands: &mut Commands,
    palette: &crate::Palette,
    assets: &AvatarAssets,
    owner: Entity,
    pilot: bool,
    position: Vec3,
) {
    let root = commands
        .spawn((
            AvatarVisual {
                owner,
                pilot,
                motion: 0,
            },
            Transform::from_translation(position).with_scale(Vec3::splat(0.65)),
            Visibility::default(),
            ChildOf(owner),
        ))
        .id();
    commands.spawn((
        AvatarFallback,
        Mesh3d(palette.sphere.clone()),
        MeshMaterial3d(palette.gold.clone()),
        Transform::from_xyz(0.0, 0.9, 0.0).with_scale(Vec3::new(0.45, 0.9, 0.45)),
        ChildOf(root),
    ));
    commands
        .spawn((
            WorldAssetRoot(assets.scene.clone()),
            Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ChildOf(root),
        ))
        .observe(ready);
}
fn ready(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    assets: Res<AvatarAssets>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    fallbacks: Query<(), With<AvatarFallback>>,
    mut players: Query<&mut AnimationPlayer>,
) {
    let Ok(parent) = parents.get(event.entity) else {
        return;
    };
    let root = parent.parent();
    for entity in children.iter_descendants(root) {
        if fallbacks.contains(entity) {
            commands.entity(entity).despawn();
        }
        if let Ok(mut player) = players.get_mut(entity) {
            player.play(assets.animations[0]).repeat();
            commands.entity(entity).insert((
                AnimationGraphHandle(assets.graph.clone()),
                AvatarPlayer { root, motion: 0 },
            ));
        }
    }
}
pub(crate) fn animate(
    assets: Res<AvatarAssets>,
    avatars: Query<&AvatarVisual>,
    mut players: Query<(&mut AnimationPlayer, &mut AvatarPlayer)>,
) {
    for (mut player, mut state) in &mut players {
        if let Ok(avatar) = avatars.get(state.root)
            && state.motion != avatar.motion
        {
            state.motion = avatar.motion.min(2);
            player.stop_all();
            player.play(assets.animations[state.motion]).repeat();
        }
    }
}
