//! Bounded, deterministic Aether circuits. Connected ports share one reservoir;
//! this is a gameplay transport model, not a volumetric fluid solver.
//!
//! Quantities use milli-units. Persistent port IDs, explicit spills and exact
//! integer apportionment keep valve edits and hull splits conservative.
use crate::PartId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const MAX_PORTS: usize = 256;
pub const MAX_LINKS: usize = 768;
pub const MAX_PORT_CAPACITY: u64 = 10_000_000;
pub const MAX_TRANSFER: u64 = MAX_PORT_CAPACITY * MAX_PORTS as u64;

pub fn vessel_ports(body: &crate::Body) -> Vec<Port> {
    ports_for_parts(body.parts())
}

pub fn ports_for_parts(parts: &[crate::Part]) -> Vec<Port> {
    let mut ports: Vec<_> = parts
        .iter()
        .filter(|p| {
            matches!(
                p.kind,
                crate::PartKind::Tank | crate::PartKind::Lift | crate::PartKind::Propeller
            )
        })
        .map(|p| Port {
            id: p.id,
            capacity: if p.kind == crate::PartKind::Tank {
                1_800_000
            } else {
                0
            },
        })
        .collect();
    ports.sort_by_key(|p| p.id);
    ports
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    pub id: PartId,
    /// Zero for consumers or junctions without their own storage.
    pub capacity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub a: PartId,
    pub b: PartId,
    pub open: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Circuit {
    pub ports: Vec<Port>,
    pub links: Vec<Link>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkSave {
    pub circuit: Circuit,
    /// Canonical per-port shares, used when an edge or a hull is split.
    pub contents: BTreeMap<PartId, u64>,
}

#[derive(Clone, Debug)]
pub struct Network {
    save: NetworkSave,
    segments: Vec<Segment>,
    memberships: BTreeMap<PartId, usize>,
}

#[derive(Clone, Debug)]
struct Segment {
    ports: Vec<PartId>,
    capacity: u64,
    amount: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TopologyChange {
    pub before: u64,
    pub after: u64,
    pub spilled: u64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NetworkError {
    #[error("Circuit trop grand ou capacité excessive.")]
    Bounds,
    #[error("Identifiant de port invalide ou répété.")]
    Port,
    #[error("Connexion invalide, répétée ou sans port.")]
    Link,
    #[error("Quantité de réserve incohérente.")]
    Contents,
    #[error("Débit demandé invalide ou consommateur répété.")]
    Request,
}

impl Circuit {
    /// Construction edits prune removed fittings and add new, isolated ports.
    /// No edges are silently installed when equipment is added.
    pub fn for_parts(&self, parts: &[crate::Part]) -> Self {
        let ports = ports_for_parts(parts);
        let ids: BTreeSet<_> = ports.iter().map(|p| p.id).collect();
        Self {
            ports,
            links: self
                .links
                .iter()
                .filter(|e| ids.contains(&e.a) && ids.contains(&e.b))
                .cloned()
                .collect(),
        }
    }

    pub fn validate_for_parts(&self, parts: &[crate::Part]) -> Result<Self, NetworkError> {
        let circuit = self.clone().normalize()?;
        if circuit.ports != ports_for_parts(parts) {
            return Err(NetworkError::Port);
        }
        Ok(circuit)
    }

    fn normalize(mut self) -> Result<Self, NetworkError> {
        if self.ports.len() > MAX_PORTS || self.links.len() > MAX_LINKS {
            return Err(NetworkError::Bounds);
        }
        self.ports.sort_by_key(|p| p.id);
        let mut ids = BTreeSet::new();
        for p in &self.ports {
            if p.id.0 == 0 || !ids.insert(p.id) {
                return Err(NetworkError::Port);
            }
            if p.capacity > MAX_PORT_CAPACITY {
                return Err(NetworkError::Bounds);
            }
        }
        let mut pairs = BTreeSet::new();
        for edge in &mut self.links {
            if edge.a > edge.b {
                std::mem::swap(&mut edge.a, &mut edge.b);
            }
            if edge.a == edge.b
                || !ids.contains(&edge.a)
                || !ids.contains(&edge.b)
                || !pairs.insert((edge.a, edge.b))
            {
                return Err(NetworkError::Link);
            }
        }
        self.links.sort_by_key(|e| (e.a, e.b));
        Ok(self)
    }
}

/// Largest-remainder allocation. Input order is stable by persistent ID; totals
/// are exact, including one-milli-unit leftovers, with no floating point drift.
pub(crate) fn allocate(total: u64, weights: &[(PartId, u64)]) -> BTreeMap<PartId, u64> {
    let sum: u64 = weights.iter().map(|(_, w)| w).sum();
    if sum == 0 {
        return weights.iter().map(|(id, _)| (*id, 0)).collect();
    }
    let mut shares = BTreeMap::new();
    let mut remainders = Vec::with_capacity(weights.len());
    let mut used = 0;
    for &(id, weight) in weights {
        let numerator = u128::from(total) * u128::from(weight);
        let share = (numerator / u128::from(sum)) as u64;
        shares.insert(id, share);
        used += share;
        remainders.push((numerator % u128::from(sum), id));
    }
    remainders.sort_by_key(|&(remainder, id)| (std::cmp::Reverse(remainder), id));
    for &(_, id) in remainders.iter().take((total - used) as usize) {
        *shares.get_mut(&id).expect("allocated port") += 1;
    }
    shares
}

impl Network {
    pub fn empty(circuit: Circuit) -> Result<Self, NetworkError> {
        Self::restore(NetworkSave {
            circuit,
            contents: BTreeMap::new(),
        })
    }

    /// Untrusted saved contents are validated before creating runtime segments.
    /// Only missing zero contents are accepted; unknown IDs cannot hide fluid.
    pub fn restore(mut save: NetworkSave) -> Result<Self, NetworkError> {
        save.circuit = save.circuit.normalize()?;
        let capacities: BTreeMap<_, _> = save
            .circuit
            .ports
            .iter()
            .map(|p| (p.id, p.capacity))
            .collect();
        if save
            .contents
            .iter()
            .any(|(id, amount)| capacities.get(id).is_none_or(|c| amount > c))
        {
            return Err(NetworkError::Contents);
        }
        let mut edges: BTreeMap<PartId, Vec<PartId>> =
            capacities.keys().map(|&id| (id, Vec::new())).collect();
        for edge in save.circuit.links.iter().filter(|e| e.open) {
            edges.get_mut(&edge.a).expect("valid edge").push(edge.b);
            edges.get_mut(&edge.b).expect("valid edge").push(edge.a);
        }
        let mut memberships = BTreeMap::new();
        let mut segments = Vec::new();
        for &start in capacities.keys() {
            if memberships.contains_key(&start) {
                continue;
            }
            let index = segments.len();
            let mut pending = vec![start];
            let mut ports = Vec::new();
            memberships.insert(start, index);
            while let Some(id) = pending.pop() {
                ports.push(id);
                for &neighbor in &edges[&id] {
                    if let std::collections::btree_map::Entry::Vacant(e) =
                        memberships.entry(neighbor)
                    {
                        e.insert(index);
                        pending.push(neighbor);
                    }
                }
            }
            ports.sort();
            segments.push(Segment {
                capacity: ports.iter().map(|id| capacities[id]).sum(),
                amount: ports
                    .iter()
                    .map(|id| save.contents.get(id).copied().unwrap_or(0))
                    .sum(),
                ports,
            });
        }
        let mut network = Self {
            save,
            segments,
            memberships,
        };
        network.redistribute();
        Ok(network)
    }

    pub fn capture(&self) -> NetworkSave {
        self.save.clone()
    }

    pub fn amount(&self) -> u64 {
        self.segments.iter().map(|s| s.amount).sum()
    }

    pub fn segment_at(&self, port: PartId) -> Option<(u64, u64)> {
        self.memberships.get(&port).map(|&i| {
            let s = &self.segments[i];
            (s.amount, s.capacity)
        })
    }

    /// Undo may restore geometry but must never restore spent reserve. Keep a
    /// proportional share in each previously isolated segment, with exact total.
    pub fn limit_total(&mut self, ceiling: u64) -> u64 {
        let before = self.amount();
        let target = before.min(ceiling);
        let weights: Vec<_> = self
            .segments
            .iter()
            .map(|s| (s.ports[0], s.amount))
            .collect();
        let shares = allocate(target, &weights);
        for segment in &mut self.segments {
            segment.amount = shares[&segment.ports[0]];
        }
        self.redistribute();
        before - target
    }

    fn redistribute(&mut self) {
        let capacities: BTreeMap<_, _> = self
            .save
            .circuit
            .ports
            .iter()
            .map(|p| (p.id, p.capacity))
            .collect();
        self.save.contents.clear();
        for segment in &self.segments {
            let weights: Vec<_> = segment
                .ports
                .iter()
                .map(|&id| (id, capacities[&id]))
                .collect();
            self.save
                .contents
                .extend(allocate(segment.amount, &weights));
        }
    }

    /// Valve cuts preserve the last uniform fill shares. Removed/reduced
    /// storage explicitly spills its fluid; removed zero-capacity pipes do not.
    /// Failed topology validation leaves both fluid and runtime state untouched.
    pub fn reconfigure(&mut self, circuit: Circuit) -> Result<TopologyChange, NetworkError> {
        let circuit = circuit.normalize()?;
        let before = self.amount();
        let contents = circuit
            .ports
            .iter()
            .map(|p| {
                (
                    p.id,
                    self.save
                        .contents
                        .get(&p.id)
                        .copied()
                        .unwrap_or(0)
                        .min(p.capacity),
                )
            })
            .collect();
        let replacement = Self::restore(NetworkSave { circuit, contents })?;
        let after = replacement.amount();
        *self = replacement;
        Ok(TopologyChange {
            before,
            after,
            spilled: before - after,
        })
    }

    /// Accepted amount is bounded by free storage. Rejected excess remains with
    /// the caller, allowing a pump/refill transaction to debit exactly once.
    pub fn fill(&mut self, port: PartId, offered: u64) -> Result<u64, NetworkError> {
        if offered > MAX_TRANSFER {
            return Err(NetworkError::Request);
        }
        let index = *self.memberships.get(&port).ok_or(NetworkError::Port)?;
        let segment = &mut self.segments[index];
        let accepted = offered.min(segment.capacity - segment.amount);
        segment.amount += accepted;
        self.redistribute();
        Ok(accepted)
    }

    /// Simultaneous bounded consumption. Starved consumers in one segment get
    /// proportional shares independent of construction/request iteration order.
    pub fn withdraw(
        &mut self,
        requests: &[(PartId, u64)],
    ) -> Result<BTreeMap<PartId, u64>, NetworkError> {
        if requests.len() > MAX_PORTS {
            return Err(NetworkError::Bounds);
        }
        let mut seen = BTreeSet::new();
        let mut grouped: BTreeMap<usize, Vec<(PartId, u64)>> = BTreeMap::new();
        for &(id, amount) in requests {
            if amount > MAX_TRANSFER || !seen.insert(id) {
                return Err(NetworkError::Request);
            }
            let &segment = self.memberships.get(&id).ok_or(NetworkError::Port)?;
            grouped.entry(segment).or_default().push((id, amount));
        }
        let mut result = BTreeMap::new();
        for (index, mut demand) in grouped {
            demand.sort_by_key(|&(id, _)| id);
            let requested: u64 = demand.iter().map(|(_, n)| n).sum();
            let segment = &mut self.segments[index];
            let used = requested.min(segment.amount);
            segment.amount -= used;
            result.extend(allocate(used, &demand));
        }
        self.redistribute();
        Ok(result)
    }

    /// A directed pump transfers only the amount the source has and the target
    /// can hold. Its per-tick allowance is supplied by the fixed-step adapter.
    /// Within a shared segment there is no transfer or fabricated throughput.
    pub fn pump(
        &mut self,
        source: PartId,
        destination: PartId,
        allowance: u64,
    ) -> Result<u64, NetworkError> {
        if allowance > MAX_TRANSFER {
            return Err(NetworkError::Request);
        }
        let &a = self.memberships.get(&source).ok_or(NetworkError::Port)?;
        let &b = self
            .memberships
            .get(&destination)
            .ok_or(NetworkError::Port)?;
        if a == b {
            return Ok(0);
        }
        let moved = allowance
            .min(self.segments[a].amount)
            .min(self.segments[b].capacity - self.segments[b].amount);
        self.segments[a].amount -= moved;
        self.segments[b].amount += moved;
        self.redistribute();
        Ok(moved)
    }
}

/// Fractional debit accumulator for leaks and fixed-step pumps. An integer rate
/// yields the same total for all tick partitions of the same elapsed duration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowMeter {
    remainder: u32,
}

/// Full circuit snapshot, including fractional debits and leak configuration.
/// A network-only snapshot is sufficient for topology studies, but cannot
/// preserve sub-unit rates across a session reload.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CircuitState {
    pub network: NetworkSave,
    pub meters: BTreeMap<PartId, FlowMeter>,
    pub leaks: BTreeMap<PartId, u64>,
    pub recharge: FlowMeter,
}
impl CircuitState {
    pub fn validate_for_body(&self, body: &crate::Body) -> Result<Network, NetworkError> {
        let network = self.validated_network()?;
        if network.save.circuit.ports != vessel_ports(body)
            || body
                .circuit_design()
                .is_some_and(|design| design != &network.save.circuit)
        {
            return Err(NetworkError::Port);
        }
        Ok(network)
    }

    pub fn for_fragment(&self, body: &crate::Body) -> Result<Self, NetworkError> {
        let canonical = self.validated_network()?.capture();
        let ids: BTreeSet<_> = body.parts().iter().map(|p| p.id).collect();
        let mut next = self.clone();
        next.network = canonical;
        next.network.circuit.ports.retain(|p| ids.contains(&p.id));
        next.network
            .circuit
            .links
            .retain(|p| ids.contains(&p.a) && ids.contains(&p.b));
        next.network.contents.retain(|id, _| ids.contains(id));
        next.meters.retain(|id, _| ids.contains(id));
        next.leaks.retain(|id, _| ids.contains(id));
        next.network = next.validate_for_body(body)?.capture();
        Ok(next)
    }

    pub fn limit_total(&mut self, ceiling: u64) -> Result<u64, NetworkError> {
        let mut network = self.validated_network()?;
        let removed = network.limit_total(ceiling);
        self.network = network.capture();
        Ok(removed)
    }

    pub fn validated_network(&self) -> Result<Network, NetworkError> {
        let network = Network::restore(self.network.clone())?;
        if !self.recharge.is_valid()
            || self
                .meters
                .iter()
                .any(|(id, meter)| network.segment_at(*id).is_none() || !meter.is_valid())
            || self.leaks.iter().any(|(id, rate)| {
                *rate == 0
                    || *rate > MAX_TRANSFER
                    || !self
                        .network
                        .circuit
                        .ports
                        .iter()
                        .any(|p| p.id == *id && p.capacity > 0)
            })
        {
            return Err(NetworkError::Contents);
        }
        Ok(network)
    }
}
impl FlowMeter {
    pub fn debit(&mut self, rate_per_second: u64, micros: u32) -> Result<u64, NetworkError> {
        if micros > 1_000_000 {
            return Err(NetworkError::Request);
        }
        self.debit_nanos(rate_per_second, micros * 1000)
    }

    pub fn is_valid(&self) -> bool {
        self.remainder < 1_000_000_000
    }

    /// Fixed-step durations retain their nanoseconds instead of truncating each
    /// 60 Hz tick to 16,666 microseconds. Long-term quantity stays conservative.
    pub fn debit_nanos(&mut self, rate_per_second: u64, nanos: u32) -> Result<u64, NetworkError> {
        if rate_per_second > MAX_TRANSFER || nanos > 1_000_000_000 || !self.is_valid() {
            return Err(NetworkError::Request);
        }
        let units = rate_per_second * u64::from(nanos) + u64::from(self.remainder);
        self.remainder = (units % 1_000_000_000) as u32;
        Ok(units / 1_000_000_000)
    }
}
