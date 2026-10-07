use crate::{app::Phase, session::GameSession};
use aether_core::{expedition::Goods, world as geography, world_geometry};
use aether_sim::Vessel;
use avian3d::prelude::*;
use bevy::prelude::*;

pub fn sync_payload(world: &mut World) {
    let game = world.resource::<GameSession>();
    let Some(active) = game.active else {
        return;
    };
    let kilograms = game.expedition.cargo_mass();
    let Some(mut vessel) = world.get_mut::<Vessel>(active) else {
        return;
    };
    if (vessel.payload - kilograms).abs() < 0.001 {
        return;
    }
    vessel.payload = kilograms;
    vessel.properties = vessel.body.mass_properties().with_payload(kilograms);
    let vessel = vessel.clone();
    aether_sim::replace_geometry(&mut world.commands(), active, &vessel);
    world.flush();
}
pub fn motor_refit(world: &mut World) {
    if !docked(world) {
        world.resource_mut::<GameSession>().notice = "La motorisation se fait au quai.".into();
        return;
    }
    let Some(active) = world.resource::<GameSession>().active else {
        return;
    };
    let Some(body) = world.get::<Vessel>(active).map(|v| v.body.clone()) else {
        return;
    };
    let mut expedition = world.resource::<GameSession>().expedition.clone();
    match expedition.motor_refit(&body) {
        Ok(blueprint) => {
            if let Some(mut vessel) = world.get_mut::<Vessel>(active)
                && vessel.body.replace(blueprint).is_ok()
            {
                world.resource_mut::<GameSession>().expedition = expedition;
                world.resource_mut::<crate::editor::Editor>().reset();
                crate::editor::refresh(world, active, vec![]);
                sync_payload(world);
                world.resource_mut::<GameSession>().notice =
                    "Deux hélices installées. Maintenez la commande d'avance pour les activer."
                        .into();
            }
        }
        Err(error) => world.resource_mut::<GameSession>().notice = error.into(),
    }
    crate::interface::rebuild(world);
}
pub fn refine(world: &mut World) {
    let Some(active) = world.resource::<GameSession>().active else {
        return;
    };
    let Some(vessel) = world.get::<Vessel>(active) else {
        return;
    };
    let (fuel, capacity) = (vessel.fuel, vessel.body.fuel_capacity());
    let mut expedition = world.resource::<GameSession>().expedition.clone();
    let result = expedition.refine(fuel, capacity).and_then(|new| {
        if let Some(circuit) = world.get::<aether_sim::aether::AetherCircuit>(active) {
            let mut next = circuit.clone();
            let available = (f64::from(capacity) * 1000.0).round() as u64 - next.remaining_milli();
            next.add_milli(240_000.min(available))
                .map_err(|_| "Le circuit ne peut pas recevoir d'Aether")?;
            let remaining = next.remaining_milli() as f32 / 1000.0;
            world.entity_mut(active).insert(next);
            Ok(remaining)
        } else {
            Ok(new)
        }
    });
    world.resource_mut::<GameSession>().notice = match result {
        Ok(new) => {
            world.resource_mut::<GameSession>().expedition = expedition;
            world.get_mut::<Vessel>(active).expect("active vessel").fuel = new;
            "Un cristal raffiné : jusqu'à 240 unités d'Aether ajoutées.".into()
        }
        Err(error) => error.into(),
    };
    crate::interface::rebuild(world);
}

pub fn observe(mut game: ResMut<GameSession>, positions: Query<&Position>) {
    let Some(entity) = game.walker.or(game.active) else {
        return;
    };
    let Ok(p) = positions.get(entity) else {
        return;
    };
    let discovered: Vec<_> = geography::islands()
        .iter()
        .filter(|i| {
            p.0.distance(i.center) < i.radius() + 130.0 && !game.expedition.visited.contains(&i.id)
        })
        .map(|i| (i.id, i.label()))
        .collect();
    for (id, name) in discovered {
        if game.expedition.discover(id) {
            game.notice = format!(
                "Découverte : {name}. Votre atlas a été complété ; prime de cartographie reçue."
            );
        }
    }
}
fn docked(world: &World) -> bool {
    world
        .resource::<GameSession>()
        .active
        .and_then(|e| world.get::<Vessel>(e))
        .is_some_and(|v| v.docked)
}
pub fn trade(world: &mut World, goods: Goods, buy: bool) {
    if !docked(world) {
        world.resource_mut::<GameSession>().notice =
            "Le commerce se fait au comptoir d'un port, une fois amarré.".into();
        return;
    }
    let checkpoint = world.resource::<GameSession>().checkpoint;
    let biome = geography::island(checkpoint).map_or(geography::Biome::Dawn, |i| i.biome);
    let result = world
        .resource_mut::<GameSession>()
        .expedition
        .trade(goods, biome, buy);
    world.resource_mut::<GameSession>().notice = match result {
        Ok(()) => format!(
            "{} : {}.",
            if buy { "Achat" } else { "Vente" },
            goods.label()
        ),
        Err(e) => e.into(),
    };
    crate::interface::rebuild(world);
}
pub fn deliver(world: &mut World) {
    if !docked(world) {
        world.resource_mut::<GameSession>().notice =
            "Amarrez-vous avant de livrer la cargaison.".into();
        return;
    }
    let port = world.resource::<GameSession>().checkpoint;
    let result = world.resource_mut::<GameSession>().expedition.deliver(port);
    world.resource_mut::<GameSession>().notice = match result {
        Ok(reward) => format!("Contrat livré : {reward} crédits. Merci, capitaine."),
        Err(e) => e.into(),
    };
    crate::interface::rebuild(world);
}
pub fn interact(world: &mut World) {
    if docked(world) && world.resource::<GameSession>().walker.is_none() {
        world.resource_mut::<NextState<Phase>>().set(Phase::Atlas);
        return;
    }
    let game = world.resource::<GameSession>();
    let Some(active) = game.active else {
        return;
    };
    let walker = game.walker;
    let Some(position) = world.get::<Position>(walker.unwrap_or(active)).map(|p| p.0) else {
        return;
    };
    // The gate has priority when the ship is inside its activation volume;
    // adjacent minerals or a passing cavewing must not prevent passage.
    let portal = geography::islands()
        .iter()
        .filter(|i| i.capital && walker.is_none())
        .find(|i| {
            world_geometry::landmarks_for(i.asset_key())
                .portal
                .is_some_and(|p| i.transform_point(p).distance(position) < 12.0 * i.scale)
        });
    let creature = world
        .query::<(&aether_sim::fauna::Resident, &Position)>()
        .iter(world)
        .filter(|_| portal.is_none())
        .filter_map(|(r, p)| {
            let distance = p.0.distance(position);
            (distance < if walker.is_some() { 22.0 } else { 35.0 }
                && aether_core::terrain::line_clear(position, p.0))
            .then_some((r.0.species, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|v| v.0);
    if let Some(species) = creature {
        let discovered = world
            .resource_mut::<GameSession>()
            .expedition
            .catalogue(species);
        world.resource_mut::<GameSession>().notice = format!(
            "{} : {}",
            species.label(),
            if discovered {
                "espèce observée, +75 crédits de recherche."
            } else {
                "déjà inscrit dans votre carnet."
            }
        );
        return;
    }
    let game = world.resource::<GameSession>();
    let closest = geography::islands()
        .iter()
        .filter(|_| portal.is_none())
        .flat_map(|i| {
            aether_core::expedition::resource_nodes(i).map(move |node| {
                let offset = position - node.position;
                let contact =
                    node.position + offset.normalize_or_zero() * node.radius.min(offset.length());
                (i.id, node.slot, contact)
            })
        })
        .filter(|(id, n, _)| !game.expedition.harvested.contains(&(id * 64 + n)))
        .map(|(id, n, p)| (id, n, p, p.distance(position)))
        .filter(|(_, _, p, distance)| {
            *distance < if walker.is_some() { 5.0 } else { 18.0 }
                && aether_core::terrain::line_clear(position, *p)
        })
        .min_by(|a, b| a.3.total_cmp(&b.3));
    if let Some((id, node, _, _)) = closest {
        let result = world
            .resource_mut::<GameSession>()
            .expedition
            .harvest(id, node);
        world.resource_mut::<GameSession>().notice = match result {
            Ok(goods) => format!("Récolte : 5 × {} ajoutés à la soute.", goods.label()),
            Err(e) => e.into(),
        };
        return;
    }
    if walker.is_none()
        && let Some(portal) = portal
    {
        let game = world.resource::<GameSession>();
        let destination = game.expedition.destination;
        let tick = world.resource::<aether_sim::SimClock>().tick;
        if tick < game.expedition.portal_ready_tick {
            world.resource_mut::<GameSession>().notice =
                "Le portail se recharge. Attendez quelques secondes.".into();
            return;
        }
        if !game.expedition.visited.contains(&destination) || destination == portal.id {
            world.resource_mut::<GameSession>().notice =
                "Choisissez dans l'atlas un autre port déjà découvert pour stabiliser le portail."
                    .into();
            return;
        }
        let fuel = world.get::<Vessel>(active).map_or(0.0, |v| v.fuel);
        if fuel < 80.0 {
            world.resource_mut::<GameSession>().notice =
                "Le passage demande 80 unités d'Aether.".into();
            return;
        }
        let rotation = geography::dock_rotation(destination);
        let target = geography::dock(destination).expect("validated destination")
            + rotation * Vec3::new(0.0, 10.0, 45.0);
        let reserve =
            if let Some(mut circuit) = world.get_mut::<aether_sim::aether::AetherCircuit>(active) {
                if let Err(error) = circuit.pay_charge(80_000) {
                    world.resource_mut::<GameSession>().notice = error.to_string();
                    return;
                }
                Some(circuit.remaining_milli() as f32 / 1000.0)
            } else {
                None
            };
        crate::controls::release_tether(world);
        aether_sim::streaming::ensure(world, destination);
        world.entity_mut(active).insert((
            aether_sim::teleport_pose(target, rotation),
            LinearVelocity::ZERO,
            AngularVelocity::ZERO,
        ));
        let mut vessel = world.get_mut::<Vessel>(active).expect("active vessel");
        vessel.fuel = reserve.unwrap_or(vessel.fuel - 80.0);
        vessel.target_altitude = target.y;
        world
            .resource_mut::<GameSession>()
            .expedition
            .portal_ready_tick = tick.saturating_add(60 * 15);
        world.resource_mut::<GameSession>().notice = format!(
            "Passage vers {}. Approchez du quai pour vous amarrer.",
            geography::port_name(destination)
        );
        world.resource_mut::<crate::camera::CameraRig>().snap = true;
        return;
    }
    world.resource_mut::<GameSession>().notice="B : récolter les cristaux et l'eau des bassins, observer une créature ou franchir un portail de capitale. Amarrez-vous pour le comptoir.".into();
}
