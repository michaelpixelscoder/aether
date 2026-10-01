use std::collections::VecDeque;

use bevy::math::Vec3;
use rayon::prelude::*;

/// One simulated point in a flexible structure.
#[derive(Clone, Copy, Debug)]
pub struct FlexibleNode {
    pub position: Vec3,
    pub previous_position: Vec3,
    pub inverse_mass: f32,
}

impl FlexibleNode {
    pub fn dynamic(position: Vec3, mass: f32) -> Self {
        assert!(mass.is_finite() && mass > 0.0, "node mass must be positive");
        Self {
            position,
            previous_position: position,
            inverse_mass: mass.recip(),
        }
    }

    pub fn fixed(position: Vec3) -> Self {
        Self {
            position,
            previous_position: position,
            inverse_mass: 0.0,
        }
    }

    #[inline]
    pub fn is_fixed(self) -> bool {
        self.inverse_mass == 0.0
    }
}

/// A distance relationship shared by ropes and fabric sheets.
#[derive(Clone, Copy, Debug)]
pub struct DistanceConstraint {
    pub a: usize,
    pub b: usize,
    pub rest_length: f32,
    pub compliance: f32,
    pub breaking_strain: f32,
    pub broken: bool,
    lambda: f32,
}

impl DistanceConstraint {
    pub fn new(a: usize, b: usize, rest_length: f32, compliance: f32) -> Self {
        assert!(a != b, "a constraint must connect two nodes");
        assert!(rest_length.is_finite() && rest_length > 0.0);
        assert!(compliance.is_finite() && compliance >= 0.0);
        Self {
            a,
            b,
            rest_length,
            compliance,
            breaking_strain: f32::INFINITY,
            broken: false,
            lambda: 0.0,
        }
    }

    #[inline]
    pub fn strain(self, nodes: &[FlexibleNode]) -> f32 {
        let length = nodes[self.a].position.distance(nodes[self.b].position);
        (length - self.rest_length) / self.rest_length
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintKind {
    Structural,
    Shear,
    Bend,
    Tether,
}

#[derive(Clone, Copy, Debug)]
pub struct FlexibleConstraint {
    pub distance: DistanceConstraint,
    pub kind: ConstraintKind,
}

#[derive(Clone, Copy, Debug)]
pub struct SolverConfig {
    pub substeps: u32,
    pub iterations: u32,
    pub damping: f32,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            substeps: 2,
            iterations: 8,
            damping: 0.015,
        }
    }
}

/// Internal cached edge-coloring of the constraint graph.
///
/// Constraints within a color never touch the same node, which means they can
/// safely be solved in parallel while colors themselves are processed in order.
/// This preserves the useful convergence behavior of Gauss-Seidel much better
/// than a fully-Jacobi update.
#[derive(Clone, Debug, Default)]
struct SolverCache {
    topology_signature: u64,
    colors: Vec<Vec<usize>>,
}

#[derive(Clone, Debug, Default)]
pub struct FlexibleGraph {
    pub nodes: Vec<FlexibleNode>,
    pub constraints: Vec<FlexibleConstraint>,

    // NOTE:
    // This is the only API-level structural change from the original type.
    // Code using FlexibleGraph::default() / add_node() / add_distance() is
    // unchanged. Code constructing FlexibleGraph with a struct literal should
    // switch to FlexibleGraph::from_parts().
    solver_cache: SolverCache,
}

impl FlexibleGraph {
    /// Convenience constructor for callers that previously used a struct literal.
    pub fn from_parts(nodes: Vec<FlexibleNode>, constraints: Vec<FlexibleConstraint>) -> Self {
        Self {
            nodes,
            constraints,
            solver_cache: SolverCache::default(),
        }
    }

    pub fn add_node(&mut self, node: FlexibleNode) -> usize {
        let index = self.nodes.len();
        self.nodes.push(node);
        self.invalidate_solver_cache();
        index
    }

    pub fn add_distance(
        &mut self,
        a: usize,
        b: usize,
        rest_length: f32,
        compliance: f32,
        kind: ConstraintKind,
    ) -> usize {
        assert!(a < self.nodes.len() && b < self.nodes.len());
        let index = self.constraints.len();
        self.constraints.push(FlexibleConstraint {
            distance: DistanceConstraint::new(a, b, rest_length, compliance),
            kind,
        });
        self.invalidate_solver_cache();
        index
    }

    /// Invalidates the cached constraint coloring.
    ///
    /// You normally do not need to call this when using the public helper
    /// methods. Call it if you directly change constraint endpoints through the
    /// public `constraints` vector.
    pub fn invalidate_solver_cache(&mut self) {
        self.solver_cache.topology_signature = 0;
        self.solver_cache.colors.clear();
    }

    /// Advances the graph using XPBD distance constraints.
    ///
    /// The public API matches the original implementation, but internally this
    /// version:
    /// - caches an edge-coloring of the constraint graph;
    /// - integrates nodes in parallel;
    /// - solves non-conflicting constraints in parallel;
    /// - precomputes the XPBD compliance time factor once per substep;
    /// - skips expensive breaking-strain checks for unbreakable constraints.
    pub fn step(&mut self, dt: f32, accelerations: &[Vec3], config: SolverConfig) {
        assert!(dt.is_finite() && dt > 0.0);
        assert!(accelerations.len() == self.nodes.len());
        assert!(config.substeps > 0 && config.iterations > 0);

        self.ensure_solver_cache();

        let sub_dt = dt / config.substeps as f32;
        let sub_dt_sq = sub_dt * sub_dt;
        let alpha_scale = sub_dt_sq.recip();
        let velocity_scale = (1.0 - config.damping).clamp(0.0, 1.0);

        for _ in 0..config.substeps {
            integrate_nodes_parallel(&mut self.nodes, accelerations, velocity_scale, sub_dt_sq);

            // XPBD multipliers are per-substep.
            self.constraints.par_iter_mut().for_each(|constraint| {
                constraint.distance.lambda = 0.0;
            });

            for _ in 0..config.iterations {
                for color in &self.solver_cache.colors {
                    solve_color_parallel(
                        &mut self.nodes,
                        &mut self.constraints,
                        color,
                        alpha_scale,
                    );
                }
            }

            // Most cloth constraints never break. Avoiding the distance/sqrt in
            // that common case is a significant saving on large sheets.
            let nodes = &self.nodes;
            self.constraints.par_iter_mut().for_each(|constraint| {
                let distance = &mut constraint.distance;
                if distance.broken || !distance.breaking_strain.is_finite() {
                    return;
                }

                let a = nodes[distance.a].position;
                let b = nodes[distance.b].position;
                let length = (a - b).length();
                let strain = (length - distance.rest_length) / distance.rest_length;

                if strain > distance.breaking_strain {
                    distance.broken = true;
                }
            });
        }
    }

    pub fn set_fixed_position(&mut self, node: usize, position: Vec3) {
        let node = &mut self.nodes[node];
        assert!(
            node.is_fixed(),
            "only fixed nodes can be moved kinematically"
        );
        node.position = position;
        node.previous_position = position;
    }

    pub fn cut_constraint(&mut self, constraint: usize) {
        self.constraints[constraint].distance.broken = true;
    }

    /// Labels connected components using only intact constraints.
    pub fn connected_components(&self) -> Vec<usize> {
        let mut labels = vec![usize::MAX; self.nodes.len()];
        let mut adjacency = vec![Vec::new(); self.nodes.len()];

        for constraint in &self.constraints {
            if constraint.distance.broken {
                continue;
            }

            adjacency[constraint.distance.a].push(constraint.distance.b);
            adjacency[constraint.distance.b].push(constraint.distance.a);
        }

        let mut component = 0;
        for start in 0..self.nodes.len() {
            if labels[start] != usize::MAX {
                continue;
            }

            labels[start] = component;
            let mut queue = VecDeque::from([start]);

            while let Some(node) = queue.pop_front() {
                for &neighbor in &adjacency[node] {
                    if labels[neighbor] == usize::MAX {
                        labels[neighbor] = component;
                        queue.push_back(neighbor);
                    }
                }
            }

            component += 1;
        }

        labels
    }

    pub fn is_finite(&self) -> bool {
        self.nodes
            .iter()
            .all(|node| node.position.is_finite() && node.previous_position.is_finite())
    }

    fn ensure_solver_cache(&mut self) {
        let signature = topology_signature(&self.nodes, &self.constraints);
        if signature == self.solver_cache.topology_signature && !self.solver_cache.colors.is_empty()
        {
            return;
        }

        self.solver_cache.colors = build_constraint_coloring(self.nodes.len(), &self.constraints);
        self.solver_cache.topology_signature = signature;
    }
}

#[inline]
fn constraint_kind_id(kind: ConstraintKind) -> u64 {
    match kind {
        ConstraintKind::Structural => 1,
        ConstraintKind::Shear => 2,
        ConstraintKind::Bend => 3,
        ConstraintKind::Tether => 4,
    }
}

/// Cheap signature used to notice direct external edits to public topology.
///
/// This is O(number of constraints), but it is only one compact integer pass per
/// frame, while the solver usually makes many more passes.
fn topology_signature(nodes: &[FlexibleNode], constraints: &[FlexibleConstraint]) -> u64 {
    // FNV-1a style rolling hash.
    let mut hash = 0xcbf29ce484222325u64;

    #[inline]
    fn mix(hash: &mut u64, value: u64) {
        *hash ^= value;
        *hash = hash.wrapping_mul(0x100000001b3);
    }

    mix(&mut hash, nodes.len() as u64);
    mix(&mut hash, constraints.len() as u64);

    for constraint in constraints {
        mix(&mut hash, constraint.distance.a as u64);
        mix(&mut hash, constraint.distance.b as u64);
        mix(&mut hash, constraint_kind_id(constraint.kind));
    }

    // Zero is reserved for "invalid cache".
    if hash == 0 { 1 } else { hash }
}

/// Greedy edge coloring.
///
/// For ordinary ropes and grid cloth this produces only a small number of
/// colors. Each color contains constraints that have no shared endpoints.
fn build_constraint_coloring(
    node_count: usize,
    constraints: &[FlexibleConstraint],
) -> Vec<Vec<usize>> {
    // A list rather than a giant bitset keeps this generic and rebuilds are rare.
    let mut node_colors: Vec<Vec<usize>> = vec![Vec::new(); node_count];
    let mut colors: Vec<Vec<usize>> = Vec::new();

    for (constraint_index, constraint) in constraints.iter().enumerate() {
        let a = constraint.distance.a;
        let b = constraint.distance.b;

        assert!(
            a < node_count && b < node_count,
            "constraint node out of range"
        );
        assert!(a != b, "a constraint must connect two different nodes");

        let mut color = 0usize;
        loop {
            let used_by_a = node_colors[a].contains(&color);
            let used_by_b = node_colors[b].contains(&color);

            if !used_by_a && !used_by_b {
                break;
            }

            color += 1;
        }

        if color == colors.len() {
            colors.push(Vec::new());
        }

        colors[color].push(constraint_index);
        node_colors[a].push(color);
        node_colors[b].push(color);
    }

    colors
}

#[inline]
fn integrate_nodes_parallel(
    nodes: &mut [FlexibleNode],
    accelerations: &[Vec3],
    velocity_scale: f32,
    dt_sq: f32,
) {
    // For small ropes Rayon overhead can cost more than it saves.
    const PARALLEL_NODE_THRESHOLD: usize = 2_048;

    if nodes.len() < PARALLEL_NODE_THRESHOLD {
        for (node, &acceleration) in nodes.iter_mut().zip(accelerations) {
            integrate_node(node, acceleration, velocity_scale, dt_sq);
        }
        return;
    }

    nodes
        .par_iter_mut()
        .zip(accelerations.par_iter().copied())
        .for_each(|(node, acceleration)| {
            integrate_node(node, acceleration, velocity_scale, dt_sq);
        });
}

#[inline(always)]
fn integrate_node(node: &mut FlexibleNode, acceleration: Vec3, velocity_scale: f32, dt_sq: f32) {
    if node.inverse_mass == 0.0 {
        node.previous_position = node.position;
        return;
    }

    let velocity = (node.position - node.previous_position) * velocity_scale;
    node.previous_position = node.position;
    node.position += velocity + acceleration * dt_sq;
}

/// Solves one edge color.
///
/// SAFETY:
/// `build_constraint_coloring()` guarantees that no two constraints in a color
/// touch the same node. Constraint indices are also unique. Therefore each
/// worker has exclusive access to its two nodes and its own constraint even
/// though raw pointers are used to express this to Rayon.
fn solve_color_parallel(
    nodes: &mut [FlexibleNode],
    constraints: &mut [FlexibleConstraint],
    color: &[usize],
    alpha_scale: f32,
) {
    const PARALLEL_CONSTRAINT_THRESHOLD: usize = 512;

    if color.len() < PARALLEL_CONSTRAINT_THRESHOLD {
        for &constraint_index in color {
            solve_constraint_by_index(nodes, constraints, constraint_index, alpha_scale);
        }
        return;
    }

    let node_ptr = nodes.as_mut_ptr() as usize;
    let constraint_ptr = constraints.as_mut_ptr() as usize;
    let node_len = nodes.len();
    let constraint_len = constraints.len();

    color.par_iter().for_each(|&constraint_index| unsafe {
        debug_assert!(constraint_index < constraint_len);

        let constraint = &mut *((constraint_ptr as *mut FlexibleConstraint).add(constraint_index));

        if constraint.distance.broken {
            return;
        }

        let a_index = constraint.distance.a;
        let b_index = constraint.distance.b;

        debug_assert!(a_index < node_len);
        debug_assert!(b_index < node_len);
        debug_assert_ne!(a_index, b_index);

        let a = &mut *((node_ptr as *mut FlexibleNode).add(a_index));
        let b = &mut *((node_ptr as *mut FlexibleNode).add(b_index));

        solve_distance_pair(a, b, &mut constraint.distance, alpha_scale);
    });
}

#[inline(always)]
fn solve_constraint_by_index(
    nodes: &mut [FlexibleNode],
    constraints: &mut [FlexibleConstraint],
    constraint_index: usize,
    alpha_scale: f32,
) {
    let constraint = &mut constraints[constraint_index];
    if constraint.distance.broken {
        return;
    }

    let a_index = constraint.distance.a;
    let b_index = constraint.distance.b;

    // Safe split borrow for the small/sequential path.
    if a_index < b_index {
        let (left, right) = nodes.split_at_mut(b_index);
        let a = &mut left[a_index];
        let b = &mut right[0];
        solve_distance_pair(a, b, &mut constraint.distance, alpha_scale);
    } else {
        let (left, right) = nodes.split_at_mut(a_index);
        let b = &mut left[b_index];
        let a = &mut right[0];
        solve_distance_pair(a, b, &mut constraint.distance, alpha_scale);
    }
}

#[inline(always)]
fn solve_distance_pair(
    a: &mut FlexibleNode,
    b: &mut FlexibleNode,
    constraint: &mut DistanceConstraint,
    alpha_scale: f32,
) {
    let delta = a.position - b.position;
    let length_sq = delta.length_squared();

    // Comparing squared lengths avoids sqrt for degenerate constraints.
    if length_sq <= f32::EPSILON * f32::EPSILON {
        return;
    }

    let weight = a.inverse_mass + b.inverse_mass;
    if weight == 0.0 {
        return;
    }

    let length = length_sq.sqrt();
    let inv_length = length.recip();

    let alpha = constraint.compliance * alpha_scale;
    let value = length - constraint.rest_length;
    let delta_lambda = (-value - alpha * constraint.lambda) / (weight + alpha);

    constraint.lambda += delta_lambda;

    let correction = delta * (delta_lambda * inv_length);

    a.position += correction * a.inverse_mass;
    b.position -= correction * b.inverse_mass;
}

#[derive(Clone, Debug)]
pub struct RopeTopology {
    pub nodes: Vec<usize>,
    pub constraints: Vec<usize>,
}

pub fn build_rope(
    graph: &mut FlexibleGraph,
    start: Vec3,
    end: Vec3,
    segments: usize,
    mass_per_node: f32,
    compliance: f32,
) -> RopeTopology {
    assert!(segments > 0);

    let rest_length = start.distance(end) / segments as f32;
    let mut nodes = Vec::with_capacity(segments + 1);

    for index in 0..=segments {
        let position = start.lerp(end, index as f32 / segments as f32);
        nodes.push(graph.add_node(FlexibleNode::dynamic(position, mass_per_node)));
    }

    let mut constraints = Vec::with_capacity(segments);
    for pair in nodes.windows(2) {
        constraints.push(graph.add_distance(
            pair[0],
            pair[1],
            rest_length,
            compliance,
            ConstraintKind::Structural,
        ));
    }

    RopeTopology { nodes, constraints }
}

#[derive(Clone, Debug)]
pub struct FabricTopology {
    pub width: usize,
    pub height: usize,
    pub nodes: Vec<usize>,
    pub structural_constraints: Vec<usize>,
    pub shear_constraints: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct FabricSpec {
    pub bottom_left: Vec3,
    pub width: usize,
    pub height: usize,
    pub spacing: f32,
    pub mass_per_node: f32,
    pub compliance: f32,
    pub include_shear: bool,
}

impl FabricTopology {
    #[inline]
    pub fn node(&self, x: usize, y: usize) -> usize {
        self.nodes[y * self.width + x]
    }
}

pub fn build_fabric(graph: &mut FlexibleGraph, spec: FabricSpec) -> FabricTopology {
    let FabricSpec {
        bottom_left,
        width,
        height,
        spacing,
        mass_per_node,
        compliance,
        include_shear,
    } = spec;

    assert!(width > 1 && height > 1 && spacing > 0.0);

    let mut nodes = Vec::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            let position = bottom_left + Vec3::new(x as f32 * spacing, y as f32 * spacing, 0.0);

            nodes.push(graph.add_node(FlexibleNode::dynamic(position, mass_per_node)));
        }
    }

    let node = |x: usize, y: usize| nodes[y * width + x];

    let horizontal_count = (width - 1) * height;
    let vertical_count = width * (height - 1);
    let mut structural_constraints = Vec::with_capacity(horizontal_count + vertical_count);

    for y in 0..height {
        for x in 0..width {
            if x + 1 < width {
                structural_constraints.push(graph.add_distance(
                    node(x, y),
                    node(x + 1, y),
                    spacing,
                    compliance,
                    ConstraintKind::Structural,
                ));
            }

            if y + 1 < height {
                structural_constraints.push(graph.add_distance(
                    node(x, y),
                    node(x, y + 1),
                    spacing,
                    compliance,
                    ConstraintKind::Structural,
                ));
            }
        }
    }

    let mut shear_constraints = if include_shear {
        Vec::with_capacity((width - 1) * (height - 1) * 2)
    } else {
        Vec::new()
    };

    if include_shear {
        let diagonal = spacing * 2.0_f32.sqrt();

        for y in 0..height - 1 {
            for x in 0..width - 1 {
                shear_constraints.push(graph.add_distance(
                    node(x, y),
                    node(x + 1, y + 1),
                    diagonal,
                    compliance,
                    ConstraintKind::Shear,
                ));

                shear_constraints.push(graph.add_distance(
                    node(x + 1, y),
                    node(x, y + 1),
                    diagonal,
                    compliance,
                    ConstraintKind::Shear,
                ));
            }
        }
    }

    FabricTopology {
        width,
        height,
        nodes,
        structural_constraints,
        shear_constraints,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: SolverConfig = SolverConfig {
        substeps: 2,
        iterations: 12,
        damping: 0.01,
    };

    #[test]
    fn rope_and_fabric_share_the_same_graph() {
        let mut graph = FlexibleGraph::default();

        let rope = build_rope(&mut graph, Vec3::ZERO, Vec3::X * 4.0, 4, 1.0, 0.0);

        let fabric = build_fabric(
            &mut graph,
            FabricSpec {
                bottom_left: Vec3::Y,
                width: 3,
                height: 2,
                spacing: 1.0,
                mass_per_node: 1.0,
                compliance: 0.0,
                include_shear: true,
            },
        );

        assert_eq!(rope.nodes.len(), 5);
        assert_eq!(rope.constraints.len(), 4);
        assert_eq!(fabric.nodes.len(), 6);
        assert_eq!(fabric.structural_constraints.len(), 7);
        assert_eq!(fabric.shear_constraints.len(), 4);
        assert_eq!(graph.nodes.len(), 11);
    }

    #[test]
    fn fixed_node_stays_in_place_and_distance_converges() {
        let mut graph = FlexibleGraph::default();
        let a = graph.add_node(FlexibleNode::fixed(Vec3::ZERO));
        let b = graph.add_node(FlexibleNode::dynamic(Vec3::X * 3.0, 1.0));

        graph.add_distance(a, b, 1.0, 0.0, ConstraintKind::Structural);

        graph.step(1.0 / 60.0, &[Vec3::ZERO; 2], CONFIG);

        assert_eq!(graph.nodes[a].position, Vec3::ZERO);
        assert!((graph.nodes[b].position.length() - 1.0).abs() < 1.0e-5);
    }

    #[test]
    fn stepping_is_deterministic() {
        let make_graph = || {
            let mut graph = FlexibleGraph::default();

            let rope = build_rope(&mut graph, Vec3::ZERO, Vec3::X * 4.0, 8, 1.0, 1.0e-6);

            graph.nodes[rope.nodes[0]] = FlexibleNode::fixed(Vec3::ZERO);
            graph
        };

        let mut a = make_graph();
        let mut b = make_graph();

        for _ in 0..120 {
            let accelerations = vec![Vec3::new(0.0, -9.81, 0.5); a.nodes.len()];

            a.step(1.0 / 60.0, &accelerations, CONFIG);
            b.step(1.0 / 60.0, &accelerations, CONFIG);
        }

        for (a, b) in a.nodes.iter().zip(&b.nodes) {
            assert_eq!(a.position, b.position);
        }

        assert!(a.is_finite());
    }

    #[test]
    fn cutting_splits_a_rope() {
        let mut graph = FlexibleGraph::default();

        let rope = build_rope(&mut graph, Vec3::ZERO, Vec3::X * 3.0, 3, 1.0, 0.0);

        graph.cut_constraint(rope.constraints[1]);

        let labels = graph.connected_components();

        assert_eq!(labels[rope.nodes[0]], labels[rope.nodes[1]]);
        assert_eq!(labels[rope.nodes[2]], labels[rope.nodes[3]]);
        assert_ne!(labels[rope.nodes[1]], labels[rope.nodes[2]]);
    }

    #[test]
    fn sixty_seconds_remain_finite() {
        let mut graph = FlexibleGraph::default();

        let fabric = build_fabric(
            &mut graph,
            FabricSpec {
                bottom_left: Vec3::ZERO,
                width: 12,
                height: 8,
                spacing: 0.4,
                mass_per_node: 1.0,
                compliance: 2.0e-6,
                include_shear: true,
            },
        );

        for node in [fabric.node(0, 7), fabric.node(11, 7)] {
            let position = graph.nodes[node].position;
            graph.nodes[node] = FlexibleNode::fixed(position);
        }

        for _ in 0..3_600 {
            let accelerations = vec![Vec3::new(0.0, -9.81, 2.0); graph.nodes.len()];

            graph.step(1.0 / 60.0, &accelerations, CONFIG);
            assert!(graph.is_finite());
        }
    }

    #[test]
    fn coloring_never_shares_nodes_inside_a_color() {
        let mut graph = FlexibleGraph::default();

        build_fabric(
            &mut graph,
            FabricSpec {
                bottom_left: Vec3::ZERO,
                width: 32,
                height: 32,
                spacing: 0.1,
                mass_per_node: 1.0,
                compliance: 1.0e-6,
                include_shear: true,
            },
        );

        graph.ensure_solver_cache();

        for color in &graph.solver_cache.colors {
            let mut seen = vec![false; graph.nodes.len()];

            for &constraint_index in color {
                let c = graph.constraints[constraint_index].distance;

                assert!(!seen[c.a]);
                assert!(!seen[c.b]);

                seen[c.a] = true;
                seen[c.b] = true;
            }
        }
    }
}
