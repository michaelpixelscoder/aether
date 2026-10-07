//! Authoritative construction actions; the panel displays real saved circuits.
use crate::{
    controls::{Action, notice},
    editor::Editor,
    interface::{button, column, text},
    session::GameSession,
};
use aether_core::{
    PartId,
    aether_network::{Circuit, Link, vessel_ports},
    edit::Edit,
};
use aether_sim::{Vessel, aether::AetherCircuit};
use aether_view::widgets::{ACCENT, INK, MUTED, PAPER};
use bevy::prelude::*;

fn active(world: &World) -> Option<Entity> {
    world
        .resource::<GameSession>()
        .active
        .filter(|e| world.get::<Vessel>(*e).is_some_and(|v| v.docked))
}
pub fn enable(world: &mut World) {
    let Some(entity) = active(world) else {
        notice(world, "Amarrez-vous pour modifier le circuit.");
        return;
    };
    let vessel = world.get::<Vessel>(entity).expect("active vessel");
    if vessel.body.circuit_design().is_some() {
        return;
    }
    let ports = vessel_ports(&vessel.body);
    let Some(root) = ports.iter().find(|p| p.capacity > 0).map(|p| p.id) else {
        notice(world, "Installez un réservoir dans l'atelier.");
        return;
    };
    let links = ports
        .iter()
        .filter(|p| p.id != root)
        .map(|p| Link {
            a: root,
            b: p.id,
            open: true,
        })
        .collect();
    crate::editor::execute(world, Edit::Circuit(Circuit { ports, links }));
    notice(
        world,
        "Distribution installée. Les vannes commandent réellement chaque équipement.",
    );
    crate::interface::rebuild(world);
}
pub fn select(world: &mut World, id: PartId) {
    let Some(entity) = active(world) else {
        return;
    };
    let Some(design) = world
        .get::<Vessel>(entity)
        .and_then(|v| v.body.circuit_design())
        .cloned()
    else {
        return;
    };
    if !design.ports.iter().any(|p| p.id == id) {
        return;
    }
    let previous = world.resource_mut::<Editor>().circuit_pick.take();
    if let Some(a) = previous.filter(|a| *a != id) {
        let mut design = design;
        if design
            .links
            .iter()
            .any(|e| (e.a == a && e.b == id) || (e.b == a && e.a == id))
        {
            notice(world, "Cette liaison existe déjà. Utilisez sa vanne.");
        } else {
            design.links.push(Link {
                a,
                b: id,
                open: true,
            });
            crate::editor::execute(world, Edit::Circuit(design));
        }
    } else {
        world.resource_mut::<Editor>().circuit_pick = Some(id);
    }
    crate::interface::rebuild(world);
}
pub fn valve(world: &mut World, a: PartId, b: PartId) {
    let Some(entity) = active(world) else {
        return;
    };
    let Some(mut design) = world
        .get::<Vessel>(entity)
        .and_then(|v| v.body.circuit_design())
        .cloned()
    else {
        return;
    };
    let Some(link) = design
        .links
        .iter_mut()
        .find(|e| (e.a == a && e.b == b) || (e.a == b && e.b == a))
    else {
        return;
    };
    link.open = !link.open;
    crate::editor::execute(world, Edit::Circuit(design));
    crate::interface::rebuild(world);
}
pub fn repair(world: &mut World, id: PartId) {
    let Some(entity) = active(world) else {
        return;
    };
    if let Some(mut circuit) = world.get_mut::<AetherCircuit>(entity) {
        match circuit.set_tank_leak(id, 0) {
            Ok(()) => notice(world, "Réservoir réparé. La fuite est arrêtée."),
            Err(e) => notice(world, e.to_string()),
        }
    }
    crate::interface::rebuild(world);
}
pub fn build(world: &mut World, root: Entity) {
    let game = world.resource::<GameSession>();
    let notice = game.notice.clone();
    let vessel = game.active.and_then(|e| world.get::<Vessel>(e)).cloned();
    let state = game
        .active
        .and_then(|e| world.get::<AetherCircuit>(e))
        .map(AetherCircuit::capture_state);
    let picked = world.resource::<Editor>().circuit_pick;
    let panel = column(
        world,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(28),
            right: px(28),
            top: px(85),
            bottom: px(25),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        INK,
    );
    text(world, panel, "ATELIER / DISTRIBUTION D'AETHER", 28.0, PAPER);
    let nav = column(
        world,
        panel,
        Node {
            column_gap: px(10),
            ..default()
        },
        Color::NONE,
    );
    button(world, nav, "Retour à l'atlas", Action::Engineering, false);
    button(world, nav, "Sauvegarder", Action::Save, false);
    button(world, nav, "Annuler", Action::Undo, false);
    button(world, nav, "Rétablir", Action::Redo, false);
    let Some(vessel) = vessel else {
        return;
    };
    text(
        world,
        panel,
        format!(
            "{} · {:.3} / {:.0} unités · {}",
            vessel.body.name(),
            vessel.fuel,
            vessel.body.fuel_capacity(),
            if vessel.docked {
                "AMARRÉ"
            } else {
                "MODIFICATIONS AU QUAI UNIQUEMENT"
            }
        ),
        16.0,
        ACCENT,
    );
    let status = text(world, panel, notice, 14.0, PAPER);
    world
        .entity_mut(status)
        .insert(crate::interface::StatusText);
    let Some(state) = state else {
        text(
            world,
            panel,
            "Le vaisseau utilise sa réserve commune. Installer la distribution raccorde les équipements existants. Les nouveaux équipements devront ensuite être raccordés.",
            16.0,
            MUTED,
        );
        if vessel.docked {
            button(
                world,
                panel,
                "Installer la distribution",
                Action::CircuitEnable,
                true,
            );
        }
        return;
    };
    text(
        world,
        panel,
        "Choisissez deux composants pour ajouter une liaison. Fermez une vanne pour isoler sa branche. Une branche sans réservoir ne produit aucune force.",
        15.0,
        MUTED,
    );
    let network = state.validated_network().expect("runtime circuit");
    let ports = column(
        world,
        panel,
        Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(8),
            row_gap: px(8),
            ..default()
        },
        Color::NONE,
    );
    for port in &state.network.circuit.ports {
        let part = vessel
            .body
            .parts()
            .iter()
            .find(|p| p.id == port.id)
            .expect("circuit port");
        let (amount, capacity) = network.segment_at(port.id).expect("runtime port");
        let row = column(
            world,
            ports,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                width: px(230),
                ..default()
            },
            Color::NONE,
        );
        button(
            world,
            row,
            &format!("{} #{}", part.kind.label(), part.id.0),
            Action::CircuitPort(part.id),
            picked == Some(part.id),
        );
        text(
            world,
            row,
            format!(
                "Branche {:.3} / {:.0}",
                amount as f64 / 1000.0,
                capacity as f64 / 1000.0
            ),
            13.0,
            MUTED,
        );
        if let Some(rate) = state.leaks.get(&part.id) {
            text(
                world,
                row,
                format!("Fuite {:.3} / s", *rate as f64 / 1000.0),
                13.0,
                ACCENT,
            );
            if vessel.docked {
                button(
                    world,
                    row,
                    "Réparer le réservoir",
                    Action::CircuitRepair(part.id),
                    false,
                );
            }
        }
    }
    text(world, panel, "VANNES", 18.0, PAPER);
    let links = column(
        world,
        panel,
        Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(8),
            row_gap: px(8),
            ..default()
        },
        Color::NONE,
    );
    for link in &state.network.circuit.links {
        button(
            world,
            links,
            &format!(
                "#{} / #{} : {}",
                link.a.0,
                link.b.0,
                if link.open { "ouverte" } else { "fermée" }
            ),
            Action::CircuitValve(link.a, link.b),
            link.open,
        );
    }
}
