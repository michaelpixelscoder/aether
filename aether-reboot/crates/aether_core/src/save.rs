use crate::{Blueprint, Body, BodyId, DomainError, MAX_BODIES, MAX_SAVE_BYTES};
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

pub const VERSION: u32 = 3;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VesselSave {
    pub id: BodyId,
    pub blueprint: Blueprint,
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub fuel: f32,
    pub trim: f32,
    /// Missing in early v1 files: recover the current altitude when loading.
    #[serde(default)]
    pub target_altitude: Option<f32>,
    pub docked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circuit: Option<crate::aether_network::CircuitState>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TetherSave {
    pub body: BodyId,
    pub anchor: u32,
    pub local_point: Vec3,
    pub length: f32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WalkerSave {
    pub position: Vec3,
    pub velocity: Vec3,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub seed: u64,
    pub tick: u64,
    pub vessels: Vec<VesselSave>,
    pub active: BodyId,
    pub checkpoint: u32,
    pub progress: u32,
    pub tether: Option<TetherSave>,
    #[serde(default)]
    pub walker: Option<WalkerSave>,
    #[serde(default)]
    pub expedition: crate::expedition::Expedition,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlueprintFile {
    pub version: u32,
    pub cell_size: f32,
    pub blueprint: Blueprint,
}
#[derive(Debug, Error)]
pub enum SaveError {
    #[error("Fichier trop volumineux (16 Mio maximum).")]
    Size,
    #[error("Format invalide : {0}")]
    Json(#[from] serde_json::Error),
    #[error("Version de sauvegarde non prise en charge.")]
    Version,
    #[error("État de session incohérent : {0}")]
    Invalid(&'static str),
    #[error(transparent)]
    Domain(#[from] DomainError),
}
impl Session {
    /// Construction undo restores the hull, never previously spent reserve.
    /// Circuits retain an exact integer budget across all restored fragments.
    pub fn limit_reserve_from(&mut self, current: &Self) -> Result<(), SaveError> {
        self.validate()?;
        current.validate()?;
        let amount = |v: &VesselSave| -> Result<u64, SaveError> {
            if let Some(state) = &v.circuit {
                Ok(state
                    .validated_network()
                    .map_err(|_| SaveError::Invalid("circuit d'Aether"))?
                    .amount())
            } else {
                Ok((f64::from(v.fuel) * 1000.0).floor() as u64)
            }
        };
        let available = current
            .vessels
            .iter()
            .map(amount)
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .sum::<u64>();
        let weights = self
            .vessels
            .iter()
            .map(|v| amount(v).map(|n| (crate::PartId(v.id.0), n)))
            .collect::<Result<Vec<_>, _>>()?;
        let target = available.min(weights.iter().map(|(_, n)| *n).sum());
        let shares = crate::aether_network::allocate(target, &weights);
        for v in &mut self.vessels {
            let share = shares[&crate::PartId(v.id.0)];
            if let Some(state) = &mut v.circuit {
                state
                    .limit_total(share)
                    .map_err(|_| SaveError::Invalid("circuit d'Aether"))?;
                v.fuel = share as f32 / 1000.0;
            } else {
                v.fuel = v.fuel.min(share as f32 / 1000.0);
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), SaveError> {
        if self.version != VERSION {
            return Err(SaveError::Version);
        }
        if self.vessels.is_empty() || self.vessels.len() > MAX_BODIES {
            return Err(SaveError::Invalid("nombre de corps"));
        }
        if crate::world::dock(self.checkpoint).is_none() || self.progress > 4 {
            return Err(SaveError::Invalid("progression"));
        }
        if !self.expedition.validate() {
            return Err(SaveError::Invalid("expédition"));
        }
        let mut ids = BTreeSet::new();
        for v in &self.vessels {
            if v.id.0 == 0 || v.id.0 > u64::MAX - 1024 || !ids.insert(v.id) {
                return Err(SaveError::Invalid("identifiants"));
            }
            let body = Body::from_blueprint(v.blueprint.clone())?;
            if body.circuit_design().is_some() != v.circuit.is_some() {
                return Err(SaveError::Invalid("construction et circuit d'Aether"));
            }
            if let Some(state) = &v.circuit {
                let network = state
                    .validate_for_body(&body)
                    .map_err(|_| SaveError::Invalid("circuit d'Aether"))?;
                if v.fuel != network.amount() as f32 / 1000.0 {
                    return Err(SaveError::Invalid("réserve du circuit d'Aether"));
                }
            }
            if !v.position.is_finite()
                || v.position.abs().max_element() > 10_000.0
                || !v.rotation.is_finite()
                || (v.rotation.length_squared() - 1.0).abs() > 0.01
                || !v.velocity.is_finite()
                || v.velocity.length() > 150.0
                || !v.angular_velocity.is_finite()
                || v.angular_velocity.length() > 20.0
            {
                return Err(SaveError::Invalid("pose ou vitesse"));
            }
            if !v.fuel.is_finite()
                || v.fuel < 0.0
                || v.fuel > body.fuel_capacity() + 0.001
                || !v.trim.is_finite()
                || v.trim.abs() > 1.5
                || v.target_altitude.is_some_and(|y| {
                    !y.is_finite()
                        || !(crate::tuning::ALTITUDE_MIN_METERS
                            ..=crate::tuning::ALTITUDE_MAX_METERS)
                            .contains(&y)
                })
            {
                return Err(SaveError::Invalid("réserve ou voile"));
            }
        }
        if !ids.contains(&self.active) {
            return Err(SaveError::Invalid("corps contrôlé absent"));
        }
        if let Some(walker) = &self.walker
            && (!walker.position.is_finite()
                || walker.position.abs().max_element() > 10_000.0
                || !walker.velocity.is_finite()
                || walker.velocity.length() > 150.0)
        {
            return Err(SaveError::Invalid("personnage"));
        }
        if let Some(t) = &self.tether {
            if !ids.contains(&t.body)
                || crate::world::anchor(t.anchor).is_none()
                || !t.length.is_finite()
                || !(0.5..=80.0).contains(&t.length)
                || !t.local_point.is_finite()
            {
                return Err(SaveError::Invalid("câble"));
            }
            let vessel = self
                .vessels
                .iter()
                .find(|v| v.id == t.body)
                .ok_or(SaveError::Invalid("corps du câble"))?;
            // The validated blueprint bounds this point. A separate bound around
            // local zero would incorrectly reject a translated construction.
            if vessel.docked
                || !vessel.blueprint.parts.iter().any(|p| {
                    p.kind == crate::PartKind::Harpoon && p.center().distance(t.local_point) < 0.01
                })
            {
                return Err(SaveError::Invalid("attache du harpon"));
            }
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, SaveError> {
        self.validate()?;
        // Compact sessions keep the full 32 × 10k-cell envelope below 16 MiB.
        let data = serde_json::to_vec(self)?;
        if data.len() > MAX_SAVE_BYTES {
            return Err(SaveError::Size);
        }
        Ok(data)
    }
    pub fn decode(data: &[u8]) -> Result<Self, SaveError> {
        if data.len() > MAX_SAVE_BYTES {
            return Err(SaveError::Size);
        }
        let mut session: Self = serde_json::from_slice(data)?;
        if matches!(session.version, 1 | 2) {
            session.version = VERSION;
        }
        session.validate()?;
        Ok(session)
    }
}
impl BlueprintFile {
    pub fn encode(body: &Body) -> Result<Vec<u8>, SaveError> {
        Ok(serde_json::to_vec_pretty(&Self {
            version: VERSION,
            cell_size: crate::CELL_SIZE,
            blueprint: body.blueprint(),
        })?)
    }
    pub fn decode(data: &[u8]) -> Result<Body, SaveError> {
        if data.len() > MAX_SAVE_BYTES {
            return Err(SaveError::Size);
        }
        let file: Self = serde_json::from_slice(data)?;
        if !(1..=VERSION).contains(&file.version)
            || (file.cell_size - crate::CELL_SIZE).abs() > f32::EPSILON
        {
            return Err(SaveError::Version);
        }
        Ok(Body::from_blueprint(file.blueprint)?)
    }
}
