//! Authored hull fittings, batched without dropping triangles or material detail.
use aether_core::{Body, naval::Kind};
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Clone)]
struct Surface {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}
#[derive(Resource)]
pub struct ShipKit {
    surfaces: [[Surface; 3]; 4],
    pub canvas: Handle<StandardMaterial>,
}
pub const MODELS: [&str; 4] = ["rail", "trim", "cabin-detail", "lantern"];
pub fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(ShipKit {
        // Three identity nodes, each with one static primitive/material.
        // verify-ship-batches.mjs enforces this authoring contract. Use Bevy's
        // original StandardMaterial subasset, preserving every PBR property.
        surfaces: MODELS.map(|name| {
            std::array::from_fn(|slot| Surface {
                mesh: assets.load(
                    GltfAssetLabel::Primitive {
                        mesh: slot,
                        primitive: 0,
                    }
                    .from_asset(format!("ship/{name}.glb")),
                ),
                material: assets.load(format!("ship/{name}.glb#Material{slot}/std")),
            })
        }),
        canvas: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.30, 0.44, 0.72),
            base_color_texture: Some(crate::materials::surface_texture(
                &assets,
                "textures/indigo-canvas.png",
                true,
            )),
            perceptual_roughness: 0.94,
            cull_mode: None,
            ..default()
        }),
    });
}

#[derive(Component)]
pub(crate) struct PendingFittings(Body);
struct SharedShip {
    key: crate::craft::GeometryKey,
    surfaces: Vec<Surface>,
}
#[derive(Resource, Default)]
pub struct ShipMeshes(Vec<SharedShip>);

pub fn spawn(commands: &mut Commands, owner: Entity, body: &Body) {
    // Polling handles a gallery spawning before GLB loading completes. A later
    // edit replaces the request; deleting its owner cancels it automatically.
    commands.entity(owner).insert(PendingFittings(body.clone()));
}

fn fitting_transform(fitting: &aether_core::naval::Fitting) -> Transform {
    let scale = match fitting.kind {
        Kind::Rail => Vec3::new(1.0, 1.10, 1.8),
        Kind::Trim => Vec3::new(1.0, 1.4, 2.1),
        Kind::Window => Vec3::new(1.0, 1.0, 2.0),
        Kind::Lantern => Vec3::splat(1.2),
    };
    Transform::from_translation(fitting.position)
        .with_rotation(fitting.rotation)
        .with_scale(scale)
}

/// Concatenate original vertices and indices after their attachment pose.
/// Bevy transforms normals by inverse scale and tangents by forward scale;
/// UVs, winding and tangent handedness stay intact (all scales are positive).
fn combine(source: &Mesh, poses: impl Iterator<Item = Transform>) -> Option<Mesh> {
    let mut batch: Option<Mesh> = None;
    for pose in poses {
        let instance = source.clone().transformed_by(pose);
        if let Some(batch) = &mut batch {
            batch
                .merge(&instance)
                .expect("identical authored vertex layouts");
        } else {
            batch = Some(instance);
        }
    }
    batch
}

pub(crate) fn prepare(
    mut commands: Commands,
    pending: Query<(Entity, &PendingFittings)>,
    live: Query<&Mesh3d, With<crate::craft::CraftVisual>>,
    kit: Res<ShipKit>,
    mut cache: ResMut<ShipMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    mut metrics: ResMut<crate::MeshMetrics>,
) {
    let live: HashSet<_> = live.iter().map(|mesh| mesh.id()).collect();
    cache
        .0
        .retain(|entry| entry.surfaces.iter().any(|s| live.contains(&s.mesh.id())));
    if pending.is_empty()
        || kit
            .surfaces
            .iter()
            .flatten()
            .any(|s| !meshes.contains(&s.mesh) || !materials.contains(&s.material))
    {
        return;
    }
    for (owner, request) in &pending {
        let started = bevy::platform::time::Instant::now();
        let key = crate::craft::GeometryKey::of(&request.0);
        let fittings = aether_core::naval::fittings(&request.0);
        let index = if let Some(index) = cache.0.iter().position(|entry| entry.key == key) {
            index
        } else {
            let mut surfaces = Vec::new();
            for (model, kit_surfaces) in kit.surfaces.iter().enumerate() {
                let poses: Vec<_> = fittings
                    .iter()
                    .filter(|f| f.kind.asset() == MODELS[model])
                    .map(fitting_transform)
                    .collect();
                for surface in kit_surfaces {
                    if let Some(batch) = combine(
                        meshes.get(&surface.mesh).expect("loaded above"),
                        poses.iter().copied(),
                    ) {
                        surfaces.push(Surface {
                            mesh: meshes.add(batch),
                            material: surface.material.clone(),
                        });
                    }
                }
            }
            cache.0.push(SharedShip { key, surfaces });
            cache.0.len() - 1
        };
        for surface in &cache.0[index].surfaces {
            commands.spawn((
                crate::craft::CraftVisual(owner),
                crate::immutable_bounds(meshes.get(&surface.mesh).expect("cached fittings mesh")),
                Mesh3d(surface.mesh.clone()),
                MeshMaterial3d(surface.material.clone()),
                Transform::default(),
                ChildOf(owner),
            ));
        }
        for fitting in fittings.iter().filter(|f| f.kind == Kind::Lantern).take(6) {
            commands.spawn((
                crate::craft::CraftVisual(owner),
                PointLight {
                    color: Color::srgb(1.0, 0.64, 0.30),
                    intensity: 2600.0,
                    range: 9.0,
                    radius: 0.12,
                    ..default()
                },
                fitting_transform(fitting),
                ChildOf(owner),
            ));
        }
        commands.entity(owner).remove::<PendingFittings>();
        metrics
            .decoration_ms
            .push_back(started.elapsed().as_secs_f64() * 1000.0);
        if metrics.decoration_ms.len() > 240 {
            metrics.decoration_ms.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn delayed_loading_shared_hulls_and_last_owner_removal() {
        let mut app = App::new();
        let mut meshes = Assets::<Mesh>::default();
        let source = meshes.add(Mesh::from(Cuboid::from_length(0.1)));
        let held_back = meshes.remove(source.id()).unwrap();
        let mut materials = Assets::<StandardMaterial>::default();
        let material = materials.add(StandardMaterial::default());
        app.insert_resource(ShipKit {
            surfaces: std::array::from_fn(|_| {
                std::array::from_fn(|_| Surface {
                    mesh: source.clone(),
                    material: material.clone(),
                })
            }),
            canvas: material,
        })
        .insert_resource(meshes)
        .insert_resource(materials)
        .init_resource::<ShipMeshes>()
        .init_resource::<crate::MeshMetrics>()
        .add_systems(Update, prepare);
        let body = aether_core::fixtures::explorer();
        let first = app
            .world_mut()
            .spawn((Transform::default(), PendingFittings(body.clone())))
            .id();
        let second = app
            .world_mut()
            .spawn((Transform::default(), PendingFittings(body)))
            .id();
        app.update();
        assert!(app.world().get::<PendingFittings>(first).is_some());
        assert!(app.world().resource::<ShipMeshes>().0.is_empty());
        app.world_mut()
            .resource_mut::<Assets<Mesh>>()
            .insert(source.id(), held_back)
            .unwrap();
        app.update();
        assert!(app.world().get::<PendingFittings>(first).is_none());
        assert!(app.world().get::<PendingFittings>(second).is_none());
        assert_eq!(app.world().resource::<ShipMeshes>().0.len(), 1);
        let handles: Vec<_> = app.world().resource::<ShipMeshes>().0[0]
            .surfaces
            .iter()
            .map(|s| s.mesh.id())
            .collect();
        for owner in [first, second] {
            let rendered: Vec<_> = app
                .world_mut()
                .query::<(
                    &crate::craft::CraftVisual,
                    &Mesh3d,
                    &bevy::camera::primitives::Aabb,
                    &bevy::camera::visibility::NoAutoAabb,
                )>()
                .iter(app.world())
                .filter(|(visual, _, _, _)| visual.0 == owner)
                .map(|(_, mesh, bounds, _)| {
                    use bevy::camera::primitives::MeshAabb;
                    let expected = app
                        .world()
                        .resource::<Assets<Mesh>>()
                        .get(mesh)
                        .unwrap()
                        .compute_aabb()
                        .unwrap();
                    assert_eq!(bounds.min(), expected.min());
                    assert_eq!(bounds.max(), expected.max());
                    mesh.id()
                })
                .collect();
            assert_eq!(rendered, handles);
        }
        app.world_mut().despawn(first);
        app.update();
        assert_eq!(app.world().resource::<ShipMeshes>().0.len(), 1);
        app.world_mut().despawn(second);
        app.update();
        assert!(app.world().resource::<ShipMeshes>().0.is_empty());
        assert_eq!(
            app.world_mut()
                .query::<&crate::craft::CraftVisual>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn batches_preserve_authored_triangles_uvs_and_scaled_frames() {
        let mut source = Mesh::from(Cuboid::from_length(1.0));
        source.generate_tangents().unwrap();
        let poses = [
            Transform::from_xyz(-2.0, 3.0, 1.0).with_scale(Vec3::new(1.0, 1.1, 1.8)),
            Transform::from_xyz(4.0, -1.0, 2.0).with_rotation(Quat::from_rotation_y(1.1)),
        ];
        let combined = combine(&source, poses.into_iter()).unwrap();
        let n = source.count_vertices();
        let xyz = |mesh: &Mesh, attribute| match mesh.attribute(attribute).unwrap() {
            VertexAttributeValues::Float32x3(v) => v.clone(),
            _ => panic!(),
        };
        let positions = xyz(&combined, Mesh::ATTRIBUTE_POSITION);
        let normals = xyz(&combined, Mesh::ATTRIBUTE_NORMAL);
        let original_positions = xyz(&source, Mesh::ATTRIBUTE_POSITION);
        let original_normals = xyz(&source, Mesh::ATTRIBUTE_NORMAL);
        let VertexAttributeValues::Float32x4(tangents) =
            combined.attribute(Mesh::ATTRIBUTE_TANGENT).unwrap()
        else {
            panic!()
        };
        let VertexAttributeValues::Float32x4(original_tangents) =
            source.attribute(Mesh::ATTRIBUTE_TANGENT).unwrap()
        else {
            panic!()
        };
        let VertexAttributeValues::Float32x2(uv) =
            combined.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!()
        };
        let VertexAttributeValues::Float32x2(original_uv) =
            source.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!()
        };
        for (index, pose) in poses.into_iter().enumerate() {
            for vertex in 0..n {
                let k = index * n + vertex;
                assert!(Vec3::from(positions[k]).abs_diff_eq(
                    pose.transform_point(Vec3::from(original_positions[vertex])),
                    1e-6
                ));
                let expected_normal =
                    pose.rotation * (Vec3::from(original_normals[vertex]) / pose.scale).normalize();
                assert!(Vec3::from(normals[k]).abs_diff_eq(expected_normal, 1e-6));
                let original_tangent = Vec4::from(original_tangents[vertex]);
                let expected_tangent =
                    pose.rotation * (original_tangent.truncate() * pose.scale).normalize();
                assert!(
                    Vec4::from(tangents[k])
                        .truncate()
                        .abs_diff_eq(expected_tangent, 1e-6)
                );
                assert_eq!(tangents[k][3], original_tangent.w);
                assert_eq!(uv[k], original_uv[vertex]);
            }
        }
        let actual: Vec<_> = combined.indices().unwrap().iter().collect();
        let original: Vec<_> = source.indices().unwrap().iter().collect();
        assert_eq!(
            actual,
            original
                .iter()
                .copied()
                .chain(original.iter().map(|i| i + n))
                .collect::<Vec<_>>()
        );
    }
}
