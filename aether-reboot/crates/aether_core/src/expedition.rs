//! Exploration state and bounded, transactional port economy.
use crate::world::{self, Biome};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Goods {
    Timber,
    Iron,
    Crystal,
    Herbs,
    Water,
    Relic,
}
impl Goods {
    pub fn unit_mass(self) -> f32 {
        [4.0, 8.0, 3.0, 1.0, 1.0, 2.0][self as usize]
    }
    pub const ALL: [Self; 6] = [
        Self::Timber,
        Self::Iron,
        Self::Crystal,
        Self::Herbs,
        Self::Water,
        Self::Relic,
    ];
    pub fn label(self) -> &'static str {
        [
            "Bois ouvragé",
            "Lingots de fer",
            "Cristaux d'Aether",
            "Plantes médicinales",
            "Eau claire",
            "Reliques anciennes",
        ][self as usize]
    }
    pub fn price(self, biome: Biome) -> u32 {
        let base = [8, 16, 28, 12, 5, 70][self as usize];
        if local_goods(biome) == self {
            base * 3 / 4
        } else {
            base + base / 3
        }
    }
}
pub fn local_goods(biome: Biome) -> Goods {
    match biome {
        Biome::Dawn | Biome::Nomad => Goods::Timber,
        Biome::Crystal | Biome::Storm => Goods::Crystal,
        Biome::Ember | Biome::Underforge => Goods::Iron,
        Biome::Frost => Goods::Water,
        Biome::Verdant => Goods::Herbs,
        Biome::Hollow => Goods::Relic,
    }
}
/// Stable slots: crystals 0..31, water sources 32..63. The export order of
/// existing crystals is preserved when additional scenery is authored.
#[derive(Clone, Copy, Debug)]
pub struct ResourceNode {
    pub slot: u32,
    pub goods: Goods,
    pub position: glam::Vec3,
    pub radius: f32,
}
pub fn resource_nodes(island: &world::WorldIsland) -> impl Iterator<Item = ResourceNode> + '_ {
    let marks = crate::world_geometry::landmarks_for(island.asset_key());
    marks
        .crystals
        .iter()
        .take(32)
        .enumerate()
        .map(|(n, p)| ResourceNode {
            slot: n as u32,
            goods: Goods::Crystal,
            position: island.transform_point(*p + glam::Vec3::Y * 2.0),
            radius: 4.5 * island.scale,
        })
        .chain(
            marks
                .pools
                .iter()
                .take(32)
                .enumerate()
                .map(|(n, p)| ResourceNode {
                    slot: 32 + n as u32,
                    goods: Goods::Water,
                    position: island.transform_point(p.position + glam::Vec3::Y * 0.5),
                    radius: p.size.x.min(p.size.z) * 0.45 * island.scale,
                }),
        )
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expedition {
    pub destination: u32,
    pub visited: BTreeSet<u32>,
    pub harvested: BTreeSet<u32>,
    pub cargo: [u32; 6],
    pub credits: u32,
    pub contracts: BTreeSet<u32>,
    pub portal_ready_tick: u64,
    #[serde(default)]
    pub observed_species: BTreeSet<crate::fauna::Species>,
}
impl Default for Expedition {
    fn default() -> Self {
        Self {
            destination: 100,
            visited: BTreeSet::from([0]),
            harvested: BTreeSet::new(),
            cargo: [12, 4, 4, 0, 12, 0],
            credits: 160,
            contracts: BTreeSet::new(),
            portal_ready_tick: 0,
            observed_species: BTreeSet::new(),
        }
    }
}
impl Expedition {
    pub fn catalogue(&mut self, species: crate::fauna::Species) -> bool {
        if !self.observed_species.insert(species) {
            return false;
        }
        self.credits = self.credits.saturating_add(75).min(10_000_000);
        true
    }
    pub fn motor_refit(&mut self, body: &crate::Body) -> Result<crate::Blueprint, &'static str> {
        if body.count_parts(crate::PartKind::Propeller) > 0 {
            return Err("Ce vaisseau possède déjà une propulsion");
        }
        let cost = [4, 4, 2, 0, 0, 0];
        if self.cargo.iter().zip(cost).any(|(have, need)| *have < need) {
            return Err("Il faut 4 bois, 4 fers et 2 cristaux pour deux hélices");
        }
        let mut blueprint = body.blueprint();
        let mut candidates: Vec<_> = body.grid().iter().map(|(c, _)| c).collect();
        candidates.sort_by_key(|c| {
            (
                c.1.abs(),
                std::cmp::Reverse(c.0.abs()),
                std::cmp::Reverse(c.2),
            )
        });
        let free_id = |parts: &[crate::Part]| {
            (1..=parts.len() as u64 + 1)
                .find(|n| parts.iter().all(|p| p.id.0 != *n))
                .expect("bounded part IDs")
        };
        let mut next = free_id(&blueprint.parts);
        let mut installed = 0;
        for cell in candidates {
            let part = crate::Part {
                id: crate::PartId(next),
                kind: crate::PartKind::Propeller,
                cell,
                quarter_turn: 0,
            };
            blueprint.parts.push(part);
            blueprint.circuit_design = body.circuit_design().map(|d| d.for_parts(&blueprint.parts));
            if crate::Body::from_blueprint(blueprint.clone()).is_ok() {
                installed += 1;
                next = free_id(&blueprint.parts);
                if installed == 2 {
                    break;
                }
            } else {
                blueprint.parts.pop();
                blueprint.circuit_design =
                    body.circuit_design().map(|d| d.for_parts(&blueprint.parts));
            }
        }
        if installed < 2 {
            return Err("Libérez deux emplacements de pont dans l'atelier");
        }
        for (have, need) in self.cargo.iter_mut().zip(cost) {
            *have -= need;
        }
        Ok(blueprint)
    }
    pub fn refine(&mut self, fuel: f32, capacity: f32) -> Result<f32, &'static str> {
        if !fuel.is_finite() || !capacity.is_finite() || fuel < 0.0 || fuel >= capacity {
            return Err("Le réservoir ne peut pas recevoir d'Aether");
        }
        if self.cargo[Goods::Crystal as usize] == 0 {
            return Err("Un cristal est nécessaire");
        }
        self.cargo[Goods::Crystal as usize] -= 1;
        Ok((fuel + 240.0).min(capacity))
    }
    pub fn cargo_mass(&self) -> f32 {
        Goods::ALL
            .iter()
            .map(|g| self.cargo[*g as usize] as f32 * g.unit_mass())
            .sum()
    }
    pub const CARGO_CAPACITY: u32 = 400;
    pub fn validate(&self) -> bool {
        world::dock(self.destination).is_some()
            && self.visited.len() <= world::islands().len() + 3
            && self.visited.iter().all(|id| world::dock(*id).is_some())
            && self.harvested.len() <= world::islands().len() * 64
            && self.harvested.iter().all(|node| {
                world::island(node / 64)
                    .is_some_and(|i| resource_nodes(i).any(|p| p.slot == node % 64))
            })
            && self.cargo.iter().all(|n| *n <= Self::CARGO_CAPACITY)
            && self.cargo.iter().sum::<u32>() <= Self::CARGO_CAPACITY
            && self.credits <= 10_000_000
            && self
                .contracts
                .iter()
                .all(|id| world::island(*id).is_some_and(|i| i.capital))
    }
    pub fn discover(&mut self, id: u32) -> bool {
        if world::dock(id).is_none() || !self.visited.insert(id) {
            return false;
        }
        self.credits = self
            .credits
            .saturating_add(if world::island(id).is_some_and(|i| i.capital) {
                120
            } else {
                15
            })
            .min(10_000_000);
        true
    }
    pub fn harvest(&mut self, island: u32, node: u32) -> Result<Goods, &'static str> {
        let spec = world::island(island).ok_or("Île inconnue")?;
        let resource = resource_nodes(spec)
            .find(|p| p.slot == node)
            .ok_or("Gisement inconnu")?;
        let key = island * 64 + node;
        if self.harvested.contains(&key) {
            return Err("Ce gisement a déjà été récolté");
        }
        if self.cargo.iter().sum::<u32>() + 5 > Self::CARGO_CAPACITY {
            return Err("La soute est pleine");
        }
        let goods = resource.goods;
        self.cargo[goods as usize] += 5;
        self.harvested.insert(key);
        Ok(goods)
    }
    pub fn trade(&mut self, goods: Goods, biome: Biome, buy: bool) -> Result<(), &'static str> {
        let index = goods as usize;
        let price = goods.price(biome);
        if buy {
            if self.credits < price {
                return Err("Crédits insuffisants");
            }
            if self.cargo.iter().sum::<u32>() >= Self::CARGO_CAPACITY {
                return Err("La soute est pleine");
            }
            self.credits -= price;
            self.cargo[index] += 1;
        } else {
            if self.cargo[index] == 0 {
                return Err("Marchandise absente de la soute");
            }
            self.cargo[index] -= 1;
            self.credits = (self.credits + price * 3 / 4).min(10_000_000);
        }
        Ok(())
    }
    pub fn deliver(&mut self, port: u32) -> Result<u32, &'static str> {
        let spec = world::island(port)
            .filter(|i| i.capital)
            .ok_or("Les contrats sont disponibles dans les capitales")?;
        if self.contracts.contains(&port) {
            return Err("Contrat déjà livré");
        }
        let goods = contract_goods(spec.biome);
        if self.cargo[goods as usize] < 8 {
            return Err("Ce contrat demande huit unités de la marchandise indiquée");
        }
        self.cargo[goods as usize] -= 8;
        let reward = goods.price(spec.biome) * 12 + 120;
        self.credits = (self.credits + reward).min(10_000_000);
        self.contracts.insert(port);
        Ok(reward)
    }
}
pub fn contract_goods(biome: Biome) -> Goods {
    match biome {
        Biome::Dawn => Goods::Crystal,
        Biome::Crystal => Goods::Iron,
        Biome::Nomad => Goods::Herbs,
        Biome::Ember => Goods::Water,
        Biome::Frost => Goods::Timber,
        Biome::Verdant => Goods::Relic,
        Biome::Storm => Goods::Iron,
        Biome::Hollow => Goods::Crystal,
        Biome::Underforge => Goods::Timber,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visible_resource_type_and_persistent_slot_match_the_harvest() {
        for island in world::islands() {
            let mut expedition = Expedition::default();
            for node in resource_nodes(island) {
                assert!(node.position.is_finite() && node.radius > 0.0 && node.slot < 64);
                let before = expedition.cargo[node.goods as usize];
                assert_eq!(expedition.harvest(island.id, node.slot), Ok(node.goods));
                assert_eq!(expedition.cargo[node.goods as usize], before + 5);
                assert!(expedition.harvest(island.id, node.slot).is_err());
            }
            assert!(expedition.validate());
            assert!(expedition.harvest(island.id, 63).is_err());
        }
    }
    #[test]
    fn refit_reuses_free_ids_when_imported_parts_have_sparse_high_ids() {
        let mut blueprint = crate::fixtures::starter().blueprint();
        for (n, p) in blueprint.parts.iter_mut().enumerate() {
            p.id = crate::PartId(900_000 + n as u64);
        }
        let body = crate::Body::from_blueprint(blueprint).unwrap();
        let next = Expedition::default().motor_refit(&body).unwrap();
        assert!(next.parts.iter().any(|p| p.id == crate::PartId(1)));
        assert!(crate::Body::from_blueprint(next).is_ok());
    }
    #[test]
    fn legacy_ship_refit_is_transactional_and_makes_actual_engines() {
        let mut e = Expedition::default();
        let old = crate::fixtures::starter();
        let result = e.motor_refit(&old).unwrap();
        let body = crate::Body::from_blueprint(result).unwrap();
        assert_eq!(body.count_parts(crate::PartKind::Propeller), 2);
        assert_eq!(e.cargo, [8, 0, 2, 0, 12, 0]);
        assert!(body.mass_properties().mass > old.mass_properties().mass + 180.0);
        let before = serde_json::to_vec(&e).unwrap();
        assert!(e.motor_refit(&body).is_err());
        assert_eq!(before, serde_json::to_vec(&e).unwrap());
    }
    #[test]
    fn refining_and_payload_conserve_resources() {
        let mut e = Expedition::default();
        let mass = e.cargo_mass();
        let crystals = e.cargo[Goods::Crystal as usize];
        assert!(e.refine(1800.0, 1800.0).is_err());
        assert_eq!(e.cargo[Goods::Crystal as usize], crystals);
        assert_eq!(e.refine(1750.0, 1800.0).unwrap(), 1800.0);
        assert_eq!(e.cargo_mass(), mass - Goods::Crystal.unit_mass());
        let before = serde_json::to_vec(&e).unwrap();
        assert!(e.refine(f32::NAN, 1800.0).is_err());
        assert_eq!(before, serde_json::to_vec(&e).unwrap());
    }
    #[test]
    fn trading_is_bounded_and_refusal_changes_nothing() {
        let mut e = Expedition {
            credits: 0,
            ..Default::default()
        };
        let before = serde_json::to_vec(&e).unwrap();
        assert!(e.trade(Goods::Crystal, Biome::Crystal, true).is_err());
        assert_eq!(before, serde_json::to_vec(&e).unwrap());
        assert!(e.trade(Goods::Crystal, Biome::Crystal, false).is_ok());
        assert!(e.validate());
        let credits = e.credits;
        assert!(e.trade(Goods::Crystal, Biome::Crystal, true).is_err());
        assert_eq!(credits, e.credits);
    }
    #[test]
    fn discovery_and_harvest_rewards_cannot_repeat_after_save_roundtrip() {
        let mut e = Expedition::default();
        assert!(e.discover(100));
        assert!(e.harvest(100, 0).is_ok());
        let mut loaded: Expedition =
            serde_json::from_slice(&serde_json::to_vec(&e).unwrap()).unwrap();
        assert!(!loaded.discover(100));
        assert!(loaded.harvest(100, 0).is_err());
        assert!(loaded.validate());
        assert!(!loaded.discover(999));
        assert!(loaded.harvest(999, 0).is_err());
    }
    #[test]
    fn contracts_consume_cargo_and_are_once_per_port() {
        let mut e = Expedition::default();
        e.cargo[Goods::Crystal as usize] = 8;
        assert!(e.deliver(100).is_ok());
        assert_eq!(e.cargo[Goods::Crystal as usize], 0);
        assert!(e.deliver(100).is_err());
    }
}
