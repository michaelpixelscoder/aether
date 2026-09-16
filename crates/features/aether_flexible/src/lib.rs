use std::collections::VecDeque;

use bevy::math::Vec3;

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

#[derive(Clone, Debug, Default)]
pub struct FlexibleGraph {
    pub nodes: Vec<FlexibleNode>,
    pub constraints: Vec<FlexibleConstraint>,
}

impl FlexibleGraph {
    pub fn add_node(&mut self, node: FlexibleNode) -> usize {
        let index = self.nodes.len();
        self.nodes.push(node);
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
        index
    }

    /// Advances the graph using XPBD distance constraints.
    pub fn step(&mut self, dt: f32, accelerations: &[Vec3], config: SolverConfig) {
        assert!(dt.is_finite() && dt > 0.0);
        assert!(accelerations.len() == self.nodes.len());
        assert!(config.substeps > 0 && config.iterations > 0);
        let sub_dt = dt / config.substeps as f32;
        let velocity_scale = (1.0 - config.damping).clamp(0.0, 1.0);

        for _ in 0..config.substeps {
            for (node, &acceleration) in self.nodes.iter_mut().zip(accelerations) {
                if node.is_fixed() {
                    node.previous_position = node.position;
                    continue;
                }
                let velocity = (node.position - node.previous_position) * velocity_scale;
                node.previous_position = node.position;
                node.position += velocity + acceleration * sub_dt * sub_dt;
            }

            for constraint in &mut self.constraints {
                constraint.distance.lambda = 0.0;
            }
            for _ in 0..config.iterations {
                for constraint in &mut self.constraints {
                    solve_distance(&mut self.nodes, &mut constraint.distance, sub_dt);
                }
            }

            for constraint in &mut self.constraints {
                if !constraint.distance.broken
                    && constraint.distance.strain(&self.nodes) > constraint.distance.breaking_strain
                {
                    constraint.distance.broken = true;
                }
            }
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
}

fn solve_distance(nodes: &mut [FlexibleNode], constraint: &mut DistanceConstraint, dt: f32) {
    if constraint.broken {
        return;
    }
    let a = nodes[constraint.a];
    let b = nodes[constraint.b];
    let delta = a.position - b.position;
    let length = delta.length();
    if length <= f32::EPSILON {
        return;
    }
    let weight = a.inverse_mass + b.inverse_mass;
    if weight == 0.0 {
        return;
    }
    let alpha = constraint.compliance / (dt * dt);
    let value = length - constraint.rest_length;
    let delta_lambda = (-value - alpha * constraint.lambda) / (weight + alpha);
    constraint.lambda += delta_lambda;
    let correction = delta / length * delta_lambda;
    nodes[constraint.a].position += correction * a.inverse_mass;
    nodes[constraint.b].position -= correction * b.inverse_mass;
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
    let mut structural_constraints = Vec::new();
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
    let mut shear_constraints = Vec::new();
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
}
