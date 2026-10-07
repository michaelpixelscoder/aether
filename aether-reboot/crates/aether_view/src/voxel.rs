use crate::Palette;
use aether_core::{
    Block, Body, Cell, Grid, PartId,
    mesh::{self, Surface},
};
#[cfg(not(target_arch = "wasm32"))]
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    platform::time::Instant,
    prelude::*,
};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

#[derive(Component, Clone)]
pub struct BodyVisual {
    pub body: Arc<Body>,
    pub dirty: Vec<Cell>,
}
impl BodyVisual {
    pub fn new(body: &Body) -> Self {
        Self {
            body: Arc::new(body.clone()),
            dirty: body.grid().chunk_coords().collect(),
        }
    }
}
#[derive(Component)]
pub struct ChunkVisual {
    pub owner: Entity,
    pub coord: Cell,
}
#[derive(Component)]
pub struct PartVisual {
    pub owner: Entity,
    pub id: PartId,
}
/// Presentation-only scheduling hint. Editing owner first, then camera proximity.
#[derive(Resource, Default)]
pub struct MeshFocus {
    pub owner: Option<Entity>,
    pub camera: Vec3,
}
#[derive(Resource, Default)]
pub struct MeshMetrics {
    pub completed: u64,
    pub discarded: u64,
    pub pending: usize,
    pub max_ms: f64,
    pub recent_ms: VecDeque<f64>,
    pub apply_ms: VecDeque<f64>,
    pub queue_to_apply_ms: VecDeque<f64>,
    pub decoration_ms: VecDeque<f64>,
}
impl MeshMetrics {
    pub fn p95(&self) -> f64 {
        let mut values: Vec<_> = self.recent_ms.iter().copied().collect();
        values.sort_by(f64::total_cmp);
        if values.is_empty() {
            0.0
        } else {
            values[(values.len() * 95 / 100).min(values.len() - 1)]
        }
    }
}
struct Request {
    owner: Entity,
    coord: Cell,
    revision: u64,
    grid: Arc<Grid>,
    queued_at: Instant,
}
struct ResultMesh {
    owner: Entity,
    coord: Cell,
    revision: u64,
    surfaces: BTreeMap<Block, Surface>,
    ms: f64,
    queued_at: Instant,
}
#[derive(Resource, Default)]
struct MeshQueue {
    pending: BTreeMap<(Entity, Cell), Request>,
    ready: Vec<ResultMesh>,
    #[cfg(not(target_arch = "wasm32"))]
    running: Vec<Task<ResultMesh>>,
    #[cfg(target_arch = "wasm32")]
    cooperative: Option<(Request, mesh::ChunkMesher, f64)>,
}
impl MeshQueue {
    fn pop_prioritized(
        &mut self,
        focus: &MeshFocus,
        positions: impl Fn(Entity) -> Option<GlobalTransform>,
    ) -> Option<Request> {
        let key = self
            .pending
            .iter()
            .min_by(|(ka, a), (kb, b)| {
                let distance = |r: &Request| {
                    let local = r.coord.center() * 16.0 + Vec3::splat(3.75);
                    positions(r.owner).map_or(f32::MAX, |t| {
                        t.transform_point(local).distance_squared(focus.camera)
                    })
                };
                (focus.owner != Some(a.owner))
                    .cmp(&(focus.owner != Some(b.owner)))
                    .then_with(|| distance(a).total_cmp(&distance(b)))
                    .then_with(|| ka.cmp(kb))
            })
            .map(|(k, _)| *k)?;
        self.pending.remove(&key)
    }
}
pub struct VoxelPlugin;
/// Schedule after authoritative edits and replacement of a session.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrepareVoxels;
impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeshQueue>()
            .init_resource::<MeshFocus>()
            .init_resource::<MeshMetrics>()
            .init_resource::<crate::craft::CraftMeshes>()
            .add_systems(Update, (enqueue, run_jobs).chain().in_set(PrepareVoxels));
    }
}
fn enqueue(
    mut commands: Commands,
    bodies: Query<(Entity, &BodyVisual), Changed<BodyVisual>>,
    parts: Query<(Entity, &PartVisual)>,
    art: Res<crate::art::ArtAssets>,
    ship: Res<crate::ship::ShipKit>,
    fittings: Query<(Entity, &crate::craft::CraftVisual, Option<&Mesh3d>)>,
    mut craft_meshes: ResMut<crate::craft::CraftMeshes>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut queue: ResMut<MeshQueue>,
    mut metrics: ResMut<MeshMetrics>,
) {
    craft_meshes.retain_live(
        &fittings
            .iter()
            .filter_map(|(_, _, mesh)| mesh.map(|m| m.id()))
            .collect(),
    );
    for (owner, visual) in &bodies {
        let decoration_started = Instant::now();
        for (entity, craft, _) in &fittings {
            if craft.0 == owner {
                commands.entity(entity).despawn();
            }
        }
        crate::craft::spawn(
            &mut commands,
            &mut meshes,
            &mut craft_meshes,
            &palette,
            &ship.canvas,
            owner,
            &visual.body,
        );
        crate::ship::spawn(&mut commands, owner, &visual.body);
        metrics
            .decoration_ms
            .push_back(decoration_started.elapsed().as_secs_f64() * 1000.0);
        if metrics.decoration_ms.len() > 240 {
            metrics.decoration_ms.pop_front();
        }
        let revision = visual.body.revision();
        let grid = Arc::new(visual.body.grid().clone());
        for request in queue.pending.values_mut().filter(|r| r.owner == owner) {
            request.grid = grid.clone();
            request.revision = revision;
            request.queued_at = Instant::now();
        }
        for &coord in &visual.dirty {
            queue.pending.insert(
                (owner, coord),
                Request {
                    owner,
                    coord,
                    revision,
                    grid: grid.clone(),
                    queued_at: Instant::now(),
                },
            );
        }
        for (entity, part) in &parts {
            if part.owner == owner {
                commands.entity(entity).despawn();
            }
        }
        for part in visual.body.parts() {
            let center = part.center();
            let rotation =
                Quat::from_rotation_y(part.quarter_turn as f32 * std::f32::consts::FRAC_PI_2);
            let mut entity = commands.spawn((
                PartVisual { owner, id: part.id },
                WorldAssetRoot(art.part(part.kind)),
                Transform::from_translation(center)
                    .with_rotation(rotation)
                    .with_scale(Vec3::splat(
                        if part.kind == aether_core::PartKind::GrandSail {
                            2.5
                        } else {
                            1.0
                        },
                    )),
                ChildOf(owner),
            ));
            if part.kind.is_sail() {
                entity
                    .insert(crate::sail::SailWind::default())
                    .observe(crate::sail::bind);
            }
            if part.kind == aether_core::PartKind::Propeller {
                entity
                    .insert(crate::mechanisms::MotorVisual::default())
                    .observe(crate::mechanisms::bind);
            }
        }
    }
}
#[cfg(any(not(target_arch = "wasm32"), test))]
fn compute(request: Request) -> ResultMesh {
    let start = Instant::now();
    let surfaces = mesh::chunk_mesh(&request.grid, request.coord);
    ResultMesh {
        owner: request.owner,
        coord: request.coord,
        revision: request.revision,
        surfaces,
        ms: start.elapsed().as_secs_f64() * 1000.0,
        queued_at: request.queued_at,
    }
}
fn run_jobs(
    mut commands: Commands,
    mut queue: ResMut<MeshQueue>,
    mut metrics: ResMut<MeshMetrics>,
    bodies: Query<&BodyVisual>,
    chunks: Query<(Entity, &ChunkVisual)>,
    palette: Res<Palette>,
    mut meshes: ResMut<Assets<Mesh>>,
    focus: Res<MeshFocus>,
    transforms: Query<&GlobalTransform>,
) {
    queue.pending.retain(|_, r| bodies.contains(r.owner));
    let mut results = std::mem::take(&mut queue.ready);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut i = 0;
        while i < queue.running.len() {
            if let Some(result) = futures_lite::future::block_on(futures_lite::future::poll_once(
                &mut queue.running[i],
            )) {
                results.push(result);
                drop(queue.running.swap_remove(i));
            } else {
                i += 1;
            }
        }
        while queue.running.len() < 4 {
            let Some(request) = queue.pop_prioritized(&focus, |e| transforms.get(e).ok().copied())
            else {
                break;
            };
            queue
                .running
                .push(AsyncComputeTaskPool::get().spawn(async move { compute(request) }));
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        if queue.cooperative.is_none()
            && let Some(request) =
                queue.pop_prioritized(&focus, |e| transforms.get(e).ok().copied())
        {
            let mesher = mesh::ChunkMesher::new(request.coord);
            queue.cooperative = Some((request, mesher, 0.0));
        }
        if let Some((request, mut mesher, mut cpu_ms)) = queue.cooperative.take() {
            let start = Instant::now();
            let mut done = false;
            // 1.5 ms frame budget, measured between bounded face masks.
            while start.elapsed().as_secs_f64() < 0.0015 {
                if mesher.step(&request.grid) {
                    done = true;
                    break;
                }
            }
            cpu_ms += start.elapsed().as_secs_f64() * 1000.0;
            if done {
                results.push(ResultMesh {
                    owner: request.owner,
                    coord: request.coord,
                    revision: request.revision,
                    surfaces: mesher.finish(),
                    ms: cpu_ms,
                    queued_at: request.queued_at,
                });
            } else {
                queue.cooperative = Some((request, mesher, cpu_ms));
            }
        }
    }
    for result in results {
        let Ok(body) = bodies.get(result.owner) else {
            continue;
        };
        if body.body.revision() != result.revision {
            metrics.discarded += 1;
            queue.pending.insert(
                (result.owner, result.coord),
                Request {
                    owner: result.owner,
                    coord: result.coord,
                    revision: body.body.revision(),
                    grid: Arc::new(body.body.grid().clone()),
                    queued_at: Instant::now(),
                },
            );
            continue;
        }
        let apply_started = Instant::now();
        for (entity, chunk) in &chunks {
            if chunk.owner == result.owner && chunk.coord == result.coord {
                commands.entity(entity).despawn();
            }
        }
        for (block, surface) in result.surfaces {
            let mesh = to_mesh(surface);
            commands.spawn((
                ChunkVisual {
                    owner: result.owner,
                    coord: result.coord,
                },
                crate::immutable_bounds(&mesh),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(palette.block(block)),
                Transform::default(),
                ChildOf(result.owner),
            ));
        }
        metrics.completed += 1;
        metrics
            .apply_ms
            .push_back(apply_started.elapsed().as_secs_f64() * 1000.0);
        metrics
            .queue_to_apply_ms
            .push_back(result.queued_at.elapsed().as_secs_f64() * 1000.0);
        if metrics.apply_ms.len() > 240 {
            metrics.apply_ms.pop_front();
            metrics.queue_to_apply_ms.pop_front();
        }
        metrics.max_ms = metrics.max_ms.max(result.ms);
        metrics.recent_ms.push_back(result.ms);
        if metrics.recent_ms.len() > 240 {
            metrics.recent_ms.pop_front();
        }
    }
    metrics.pending = queue.pending.len();
    #[cfg(not(target_arch = "wasm32"))]
    {
        metrics.pending += queue.running.len();
    }
    #[cfg(target_arch = "wasm32")]
    {
        metrics.pending += usize::from(queue.cooperative.is_some());
    }
}
pub fn to_mesh(surface: Surface) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, surface.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, surface.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, surface.uvs)
    .with_inserted_indices(Indices::U32(surface.indices));
    mesh.generate_tangents()
        .expect("greedy faces have nondegenerate UVs");
    mesh
}

/// Our decorative faces have four private vertices, a constant normal and an
/// affine UV projection. Their tangent basis needs no cross-face welding.
/// Imported/smooth meshes keep Bevy's general MikkTSpace implementation.
pub(crate) fn to_flat_mesh(surface: Surface) -> Mesh {
    assert_eq!(surface.positions.len() % 4, 0);
    assert_eq!(surface.indices.len(), surface.positions.len() / 4 * 6);
    let mut tangents = Vec::with_capacity(surface.positions.len());
    for base in (0..surface.positions.len()).step_by(4) {
        let p = |i| Vec3::from_array(surface.positions[base + i]);
        let uv = |i| Vec2::from_array(surface.uvs[base + i]);
        let n = Vec3::from_array(surface.normals[base]);
        // Chamfer corners use a repeated vertex. Reversing a face can put the
        // zero-area triangle first, so take the nondegenerate half.
        let (a, b) = if (p(1) - p(0)).cross(p(2) - p(0)).length_squared() > 1e-16 {
            (1, 2)
        } else {
            (2, 3)
        };
        let (e1, e2) = (p(a) - p(0), p(b) - p(0));
        let (d1, d2) = (uv(a) - uv(0), uv(b) - uv(0));
        let det = d1.x * d2.y - d1.y * d2.x;
        assert!(det.abs() > 1e-12, "flat face requires nondegenerate UVs");
        let t = (e1 * d2.y - e2 * d1.y) / det;
        let t = (t - n * n.dot(t)).normalize();
        // MikkTSpace encodes UV orientation, then Bevy negates its sign.
        // Deriving it from the supplied normal would differ on inward winding.
        let sign = -det.signum();
        tangents.extend([t.extend(sign).to_array(); 4]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, surface.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, surface.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, surface.uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, tangents)
    .with_inserted_indices(Indices::U32(surface.indices))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(owner: Entity, coord: Cell, body: &Body) -> Request {
        Request {
            owner,
            coord,
            revision: body.revision(),
            grid: Arc::new(body.grid().clone()),
            queued_at: Instant::now(),
        }
    }
    #[test]
    fn editing_priority_then_nearest_chunk_and_latest_revision_wins() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Palette>()
            .init_resource::<MeshQueue>()
            .init_resource::<MeshMetrics>()
            .init_resource::<MeshFocus>()
            .add_systems(Update, run_jobs);
        let mut body = aether_core::fixtures::solid([1, 1, 1], Block::Wood);
        let owner = app.world_mut().spawn(BodyVisual::new(&body)).id();
        let other = app.world_mut().spawn_empty().id();
        let mut queue = MeshQueue::default();
        for (e, c) in [
            (owner, Cell(8, 0, 0)),
            (other, Cell(1, 0, 0)),
            (other, Cell(0, 0, 0)),
        ] {
            queue.pending.insert((e, c), request(e, c, &body));
        }
        let focus = MeshFocus {
            owner: Some(owner),
            camera: Vec3::ZERO,
        };
        assert_eq!(
            queue
                .pop_prioritized(&focus, |_| Some(GlobalTransform::IDENTITY))
                .unwrap()
                .owner,
            owner
        );
        assert_eq!(
            queue
                .pop_prioritized(&focus, |_| Some(GlobalTransform::IDENTITY))
                .unwrap()
                .coord,
            Cell(0, 0, 0)
        );
        let stale = compute(request(owner, Cell(0, 0, 0), &body));
        aether_core::edit::History::default()
            .apply(
                &mut body,
                aether_core::edit::Edit::Set(Cell(1, 0, 0), Block::Wood),
            )
            .unwrap();
        let fresh = compute(request(owner, Cell(0, 0, 0), &body));
        app.world_mut()
            .entity_mut(owner)
            .insert(BodyVisual::new(&body));
        app.world_mut().resource_mut::<MeshQueue>().ready = vec![fresh, stale];
        app.update();
        let metrics = app.world().resource::<MeshMetrics>();
        assert_eq!((metrics.completed, metrics.discarded), (1, 1));
        let chunks: Vec<_> = app
            .world_mut()
            .query::<(
                &ChunkVisual,
                &Mesh3d,
                &bevy::camera::primitives::Aabb,
                &bevy::camera::visibility::NoAutoAabb,
            )>()
            .iter(app.world())
            .collect();
        assert_eq!(chunks.len(), 1);
        // The accepted edit extends the original bound from .25 to .75 m.
        // A stale request must not restore the smaller culling volume.
        assert_eq!(chunks[0].2.min().x, -0.25);
        assert_eq!(chunks[0].2.max().x, 0.75);
        let mesh = app
            .world()
            .resource::<Assets<Mesh>>()
            .get(chunks[0].1.id())
            .unwrap();
        let bevy::mesh::VertexAttributeValues::Float32x3(points) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        else {
            panic!("positions")
        };
        assert_eq!(
            points
                .iter()
                .map(|p| p[0])
                .fold(f32::NEG_INFINITY, f32::max),
            0.75
        );
    }
}
