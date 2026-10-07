//! Cooperative placement search for authored conduits, independent of rendering.
//!
//! The polyline uses a 125 mm lattice, cardinal sockets and at least 500 mm
//! between bends. A conservative 280 mm clearance contains every R69 module's
//! measured envelope, including the valve wheel (258 mm corner bound), with
//! at least 20 mm margin. Hull cells and equipment
//! footprints are closed obstacles. This is a placement envelope, never a solid
//! physics collider substituted for a hollow pipe. Socket poses must still come
//! from the authored equipment contract before installing routes in gameplay.
use crate::{Body, Cell};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
    sync::Arc,
};
use thiserror::Error;

pub const LATTICE_MM: i32 = 125;
pub const CLEARANCE_MM: i32 = 280;
pub const MAX_STATES: usize = 16_384;
pub const MAX_WORK_PER_ADVANCE: usize = 256;
const MAX_OBSTACLES: usize = 65_536;
const MAX_PATH_POINTS: usize = 1025;
const STRAIGHT_RUN: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point(pub i32, pub i32, pub i32);
impl Point {
    pub fn millimetres(self) -> Option<[i32; 3]> {
        Some([
            self.0.checked_mul(LATTICE_MM)?,
            self.1.checked_mul(LATTICE_MM)?,
            self.2.checked_mul(LATTICE_MM)?,
        ])
    }
    fn coordinates(self) -> [i32; 3] {
        [self.0, self.1, self.2]
    }
    fn step(self, direction: Direction) -> Self {
        let d = direction.delta();
        Self(self.0 + d.0, self.1 + d.1, self.2 + d.2)
    }
    fn distance(self, other: Self) -> u32 {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1) + self.2.abs_diff(other.2)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}
impl Direction {
    const ALL: [Self; 6] = [
        Self::PosX,
        Self::NegX,
        Self::PosY,
        Self::NegY,
        Self::PosZ,
        Self::NegZ,
    ];
    pub fn opposite(self) -> Self {
        match self {
            Self::PosX => Self::NegX,
            Self::NegX => Self::PosX,
            Self::PosY => Self::NegY,
            Self::NegY => Self::PosY,
            Self::PosZ => Self::NegZ,
            Self::NegZ => Self::PosZ,
        }
    }
    fn delta(self) -> Point {
        match self {
            Self::PosX => Point(1, 0, 0),
            Self::NegX => Point(-1, 0, 0),
            Self::PosY => Point(0, 1, 0),
            Self::NegY => Point(0, -1, 0),
            Self::PosZ => Point(0, 0, 1),
            Self::NegZ => Point(0, 0, -1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Socket {
    pub point: Point,
    /// Axis pointing out of the equipment, in the hull's local frame.
    pub outward: Direction,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RouteError {
    #[error("Enveloppe de routage ou budget invalide.")]
    Bounds,
    #[error("Le raccord est bloqué ou sort de l'enveloppe de routage.")]
    Socket,
}

/// Immutable snapshot; shared by successive jobs without cloning all obstacles.
pub struct Space {
    blocked: BTreeSet<Cell>,
    low: Point,
    high: Point,
}
impl Space {
    pub fn from_body(body: &Body) -> Result<Self, RouteError> {
        let mut blocked: BTreeSet<_> = body.grid().iter().map(|(c, _)| c).collect();
        for part in body.parts() {
            for cell in part.footprint() {
                if blocked.len() >= MAX_OBSTACLES {
                    return Err(RouteError::Bounds);
                }
                blocked.insert(cell);
            }
        }
        let mut low = [i32::MAX; 3];
        let mut high = [i32::MIN; 3];
        for cell in &blocked {
            for (axis, value) in [cell.0, cell.1, cell.2].into_iter().enumerate() {
                low[axis] = low[axis].min(value * 4 - 16);
                high[axis] = high[axis].max(value * 4 + 16);
            }
        }
        if (0..3).any(|axis| {
            low[axis] < -131_072 || high[axis] > 131_072 || high[axis] - low[axis] > 640
        }) {
            return Err(RouteError::Bounds);
        }
        Ok(Self {
            blocked,
            low: Point(low[0], low[1], low[2]),
            high: Point(high[0], high[1], high[2]),
        })
    }
    fn contains(&self, point: Point) -> bool {
        point
            .coordinates()
            .into_iter()
            .zip(self.low.coordinates())
            .zip(self.high.coordinates())
            .all(|((v, low), high)| (low..=high).contains(&v))
    }
    pub fn is_free(&self, point: Point) -> bool {
        // Validate before conversion: arbitrary i32 points cannot overflow.
        if !self.contains(point) {
            return false;
        }
        let p = point.millimetres().expect("bounded lattice coordinates");
        let nearest = p.map(|v| (v + 250).div_euclid(500));
        for x in nearest[0] - 1..=nearest[0] + 1 {
            for y in nearest[1] - 1..=nearest[1] + 1 {
                for z in nearest[2] - 1..=nearest[2] + 1 {
                    if !self.blocked.contains(&Cell(x, y, z)) {
                        continue;
                    }
                    let distance: i64 = p
                        .into_iter()
                        .zip([x, y, z])
                        .map(|(v, c)| i64::from(((v - c * 500).abs() - 250).max(0)))
                        .map(|d| d * d)
                        .sum();
                    if distance <= i64::from(CLEARANCE_MM).pow(2) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    /// Endpoints and turns only, in hull-local lattice units.
    pub points: Vec<Point>,
    /// Polyline length before trimming corners for curved elbows.
    pub polyline_mm: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Searching,
    Found(Route),
    NoPath,
    Limit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct State {
    point: Point,
    heading: Direction,
    run: u8,
}

pub struct Search {
    space: Arc<Space>,
    goal: Socket,
    start: State,
    frontier: BinaryHeap<Reverse<(u32, u32, State)>>,
    costs: BTreeMap<State, u32>,
    previous: BTreeMap<State, State>,
    max_states: usize,
    status: Status,
    expanded: usize,
}
impl Search {
    pub fn new(
        space: Arc<Space>,
        start: Socket,
        goal: Socket,
        max_states: usize,
    ) -> Result<Self, RouteError> {
        if !(1..=MAX_STATES).contains(&max_states) || start.point == goal.point {
            return Err(RouteError::Bounds);
        }
        if !space.is_free(start.point) || !space.is_free(goal.point) {
            return Err(RouteError::Socket);
        }
        let start = State {
            point: start.point,
            heading: start.outward,
            run: 0,
        };
        Ok(Self {
            space,
            goal,
            start,
            frontier: BinaryHeap::from([Reverse((
                start.point.distance(goal.point) * 10,
                0,
                start,
            ))]),
            costs: BTreeMap::from([(start, 0)]),
            previous: BTreeMap::new(),
            max_states,
            status: Status::Searching,
            expanded: 0,
        })
    }
    pub fn expanded_states(&self) -> usize {
        self.expanded
    }
    pub fn stored_states(&self) -> usize {
        self.costs.len()
    }
    pub fn status(&self) -> &Status {
        &self.status
    }
    /// At most 256 queue pops per call, including stale entries. The caller can
    /// cancel simply by dropping the job; no hull or network has been mutated.
    pub fn advance(&mut self, requested_work: usize) -> &Status {
        if self.status != Status::Searching {
            return &self.status;
        }
        for _ in 0..requested_work.min(MAX_WORK_PER_ADVANCE) {
            let Some(Reverse((_, cost, current))) = self.frontier.pop() else {
                self.status = Status::NoPath;
                break;
            };
            if self.costs.get(&current) != Some(&cost) {
                continue;
            }
            self.expanded += 1;
            if current.point == self.goal.point
                && current.heading == self.goal.outward.opposite()
                && current.run >= STRAIGHT_RUN
            {
                self.status = self.reconstruct(current);
                break;
            }
            for direction in Direction::ALL {
                if (current.run < STRAIGHT_RUN && direction != current.heading)
                    || direction == current.heading.opposite()
                {
                    continue;
                }
                let point = current.point.step(direction);
                if !self.space.is_free(point) {
                    continue;
                }
                let next = State {
                    point,
                    heading: direction,
                    run: if direction == current.heading {
                        (current.run + 1).min(STRAIGHT_RUN)
                    } else {
                        1
                    },
                };
                // Heading/run belong to the state: discarding them would erase
                // valid approaches or allow elbows too close to their sockets.
                let next_cost = cost + 10 + u32::from(direction != current.heading) * 8;
                if self.costs.get(&next).is_some_and(|old| *old <= next_cost) {
                    continue;
                }
                if (!self.costs.contains_key(&next) && self.costs.len() >= self.max_states)
                    || self.frontier.len() >= self.max_states * 6
                {
                    self.status = Status::Limit;
                    return &self.status;
                }
                self.costs.insert(next, next_cost);
                self.previous.insert(next, current);
                self.frontier.push(Reverse((
                    next_cost + point.distance(self.goal.point) * 10,
                    next_cost,
                    next,
                )));
            }
        }
        &self.status
    }
    fn reconstruct(&self, mut current: State) -> Status {
        let mut points = vec![current.point];
        while current != self.start {
            if points.len() >= MAX_PATH_POINTS {
                return Status::Limit;
            }
            current = self.previous[&current];
            points.push(current.point);
        }
        points.reverse();
        let polyline_mm = (points.len() - 1) as u32 * LATTICE_MM as u32;
        let mut corners = vec![points[0]];
        for triple in points.windows(3) {
            let delta = |a: Point, b: Point| Point(b.0 - a.0, b.1 - a.1, b.2 - a.2);
            if delta(triple[0], triple[1]) != delta(triple[1], triple[2]) {
                corners.push(triple[1]);
            }
        }
        corners.push(*points.last().expect("nonempty route"));
        Status::Found(Route {
            points: corners,
            polyline_mm,
        })
    }
}
