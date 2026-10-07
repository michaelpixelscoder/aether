//! Physical circuit adapter. Distribution is installed explicitly in the dock
//! workshop; older vessels retain their established common reservoir.
use aether_core::{
    Body, PartId, PartKind,
    aether_network::{
        Circuit, CircuitState, FlowMeter, Link, Network, NetworkError, TopologyChange,
    },
    tuning,
};
use bevy::prelude::Component;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Component, Clone, Debug)]
pub struct AetherCircuit {
    network: Network,
    meters: BTreeMap<PartId, FlowMeter>,
    leaks: BTreeMap<PartId, u64>,
    recharge: FlowMeter,
}

#[derive(Clone, Debug, Default)]
pub struct Burn {
    pub fractions: BTreeMap<PartId, f32>,
    pub used_milli: u64,
    pub leaked_milli: u64,
}

fn validated(body: &Body, circuit: Circuit) -> Result<Circuit, NetworkError> {
    circuit.validate_for_parts(body.parts())
}
fn nanos(duration: Duration) -> Result<u32, NetworkError> {
    if duration > Duration::from_secs(1) {
        return Err(NetworkError::Request);
    }
    Ok(duration.as_nanos() as u32)
}

impl AetherCircuit {
    pub fn new(body: &Body, circuit: Circuit, fuel: f32) -> Result<Self, NetworkError> {
        if !fuel.is_finite() || fuel < 0.0 || fuel > body.fuel_capacity() {
            return Err(NetworkError::Contents);
        }
        let mut supply = Self {
            network: Network::empty(validated(body, circuit)?)?,
            meters: BTreeMap::new(),
            leaks: BTreeMap::new(),
            recharge: FlowMeter::default(),
        };
        let offered = (f64::from(fuel) * 1000.0).round() as u64;
        if supply.refill(offered)? != offered {
            return Err(NetworkError::Contents);
        }
        Ok(supply)
    }

    /// Explicit laboratory manifold: every existing consumer is attached to the
    /// first tank. This does not silently install piping in a saved construction.
    pub fn manifold(body: &Body, fuel: f32) -> Result<Self, NetworkError> {
        let ports = aether_core::aether_network::vessel_ports(body);
        let first = ports.iter().find(|p| p.capacity > 0).map(|p| p.id);
        let links = first.map_or_else(Vec::new, |root| {
            ports
                .iter()
                .filter(|p| p.id != root)
                .map(|p| Link {
                    a: root,
                    b: p.id,
                    open: true,
                })
                .collect()
        });
        Self::new(body, Circuit { ports, links }, fuel)
    }

    pub fn capture(&self) -> aether_core::aether_network::NetworkSave {
        self.network.capture()
    }
    pub fn capture_state(&self) -> CircuitState {
        CircuitState {
            network: self.network.capture(),
            meters: self.meters.clone(),
            leaks: self.leaks.clone(),
            recharge: self.recharge,
        }
    }
    pub fn restore(body: &Body, state: CircuitState) -> Result<Self, NetworkError> {
        let network = state.validate_for_body(body)?;
        Ok(Self {
            network,
            meters: state.meters,
            leaks: state.leaks,
            recharge: state.recharge,
        })
    }
    pub fn remaining_milli(&self) -> u64 {
        self.network.amount()
    }
    /// When distribution is uninstalled, the old scalar reserve must not round
    /// above the authoritative quantity (large fleets have coarse f32 steps).
    pub fn conservative_common_fuel(&self) -> f32 {
        let amount = self.remaining_milli();
        let mut fuel = (amount as f64 / 1000.0) as f32;
        if f64::from(fuel) * 1000.0 > amount as f64 {
            fuel = f32::from_bits(fuel.to_bits() - 1);
        }
        fuel
    }
    /// Vessel-wide services (capital portals) debit all storage proportionally.
    /// A rejected charge changes neither contents nor fractional flow meters.
    pub fn pay_charge(&mut self, amount: u64) -> Result<(), NetworkError> {
        if amount > aether_core::aether_network::MAX_TRANSFER {
            return Err(NetworkError::Request);
        }
        let before = self.network.amount();
        if amount > before {
            return Err(NetworkError::Contents);
        }
        self.network.limit_total(before - amount);
        Ok(())
    }
    pub fn reconcile_body(&mut self, body: &Body) -> Result<TopologyChange, NetworkError> {
        let design = body
            .circuit_design()
            .cloned()
            .unwrap_or_else(|| self.network.capture().circuit.for_parts(body.parts()));
        self.reconfigure(body, design)
    }

    /// Invalid offers leave the circuit untouched.
    pub fn add_milli(&mut self, offered: u64) -> Result<u64, NetworkError> {
        if offered > aether_core::aether_network::MAX_TRANSFER {
            return Err(NetworkError::Request);
        }
        let mut next = self.clone();
        let accepted = next.refill(offered)?;
        *self = next;
        Ok(accepted)
    }
    pub fn reconfigure(
        &mut self,
        body: &Body,
        circuit: Circuit,
    ) -> Result<TopologyChange, NetworkError> {
        let circuit = validated(body, circuit)?;
        let change = self.network.reconfigure(circuit)?;
        let current = self.network.capture();
        self.meters
            .retain(|id, _| current.contents.contains_key(id));
        self.leaks.retain(|id, _| {
            current
                .circuit
                .ports
                .iter()
                .any(|p| p.id == *id && p.capacity > 0)
        });
        Ok(change)
    }

    /// A tank leak has an explicit bounded rate. Zero repairs it. Ordinary
    /// consumption and leaks compete simultaneously for the same finite supply.
    pub fn set_tank_leak(&mut self, id: PartId, rate_milli: u64) -> Result<(), NetworkError> {
        if rate_milli > aether_core::aether_network::MAX_TRANSFER {
            return Err(NetworkError::Request);
        }
        if !self
            .network
            .capture()
            .circuit
            .ports
            .iter()
            .any(|p| p.id == id && p.capacity > 0)
        {
            return Err(NetworkError::Port);
        }
        if rate_milli == 0 {
            self.leaks.remove(&id);
        } else {
            self.leaks.insert(id, rate_milli);
        }
        Ok(())
    }

    fn refill(&mut self, offered: u64) -> Result<u64, NetworkError> {
        let tanks: Vec<_> = self
            .network
            .capture()
            .circuit
            .ports
            .iter()
            .filter(|p| p.capacity > 0)
            .map(|p| p.id)
            .collect();
        let mut accepted = 0;
        for tank in tanks {
            accepted += self.network.fill(tank, offered - accepted)?;
        }
        Ok(accepted)
    }

    pub fn dock_recharge(&mut self, duration: Duration) -> Result<u64, NetworkError> {
        let mut next = self.clone();
        let offered = next.recharge.debit_nanos(
            (tuning::RECHARGE_AETHER_PER_SECOND * 1000.0).round() as u64,
            nanos(duration)?,
        )?;
        let accepted = next.refill(offered)?;
        *self = next;
        Ok(accepted)
    }

    /// Rates are resolved once to milli-units/second; fractional debits are
    /// accumulated using the actual fixed duration. Invalid input is atomic.
    pub fn burn(
        &mut self,
        body: &Body,
        duration: Duration,
        lift_newtons: f32,
        throttle: f32,
    ) -> Result<Burn, NetworkError> {
        if !lift_newtons.is_finite()
            || lift_newtons < 0.0
            || !throttle.is_finite()
            || throttle.abs() > 1.0
            || lift_newtons
                > body.count_parts(PartKind::Lift) as f32 * tuning::LIFT_NEWTONS_PER_PART
        {
            return Err(NetworkError::Request);
        }
        validated(body, self.network.capture().circuit)?;
        let duration = nanos(duration)?;
        let mut next = self.clone();
        let count = body.count_parts(PartKind::Lift) as f32;
        let lift_rate = if lift_newtons > 0.0 && count > 0.0 {
            (tuning::IDLE_AETHER_PER_SECOND + lift_newtons * tuning::AETHER_PER_NEWTON_SECOND)
                / count
        } else {
            0.0
        };
        let mut requested = BTreeMap::new();
        let mut before = BTreeMap::new();
        for part in body
            .parts()
            .iter()
            .filter(|p| matches!(p.kind, PartKind::Lift | PartKind::Propeller))
        {
            let rate = if part.kind == PartKind::Lift {
                lift_rate
            } else {
                throttle.abs() * tuning::MOTOR_AETHER_PER_SECOND
            };
            let amount = next
                .meters
                .entry(part.id)
                .or_default()
                .debit_nanos((rate * 1000.0).round() as u64, duration)?;
            before.insert(
                part.id,
                (
                    rate,
                    next.network.segment_at(part.id).expect("validated port").0,
                ),
            );
            requested.insert(part.id, amount);
        }
        for (&id, &rate) in &next.leaks {
            let amount = next
                .meters
                .entry(id)
                .or_default()
                .debit_nanos(rate, duration)?;
            requested.insert(id, amount);
        }
        let requests: Vec<_> = requested.iter().map(|(&id, &n)| (id, n)).collect();
        let delivered = next.network.withdraw(&requests)?;
        let fractions = before
            .into_iter()
            .map(|(id, (rate, available))| {
                let requested = requested[&id];
                let ratio = if requested > 0 {
                    delivered[&id] as f32 / requested as f32
                } else if rate == 0.0 || available > 0 {
                    1.0
                } else {
                    0.0
                };
                (id, ratio)
            })
            .collect();
        let result = Burn {
            fractions,
            used_milli: delivered
                .iter()
                .filter(|(id, _)| !next.leaks.contains_key(id))
                .map(|(_, &n)| n)
                .sum(),
            leaked_milli: delivered
                .iter()
                .filter(|(id, _)| next.leaks.contains_key(id))
                .map(|(_, &n)| n)
                .sum(),
        };
        *self = next;
        Ok(result)
    }
}
