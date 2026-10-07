use crate::{
    controls::Action,
    interface::{button, column, text},
    session::GameSession,
};
use aether_core::{
    expedition::{Goods, contract_goods},
    world as geography,
};
use aether_view::widgets::{ACCENT, INK, MUTED, PAPER};
use bevy::prelude::*;

fn point(p: Vec3) -> Vec2 {
    Vec2::new(
        (p.x + 4100.0) / 8200.0 * 710.0,
        (p.z + 6300.0) / 7900.0 * 440.0,
    )
}
pub fn build(world: &mut World, root: Entity) {
    if world.resource::<crate::editor::Editor>().circuit_panel {
        crate::engineering::build(world, root);
        return;
    }
    let game = world.resource::<GameSession>();
    let expedition = game.expedition.clone();
    let notice = game.notice.clone();
    let checkpoint = game.checkpoint;
    let vessel = game.active.and_then(|e| world.get::<aether_sim::Vessel>(e));
    let docked = vessel.is_some_and(|v| v.docked);
    let needs_motor =
        vessel.is_some_and(|v| v.body.count_parts(aether_core::PartKind::Propeller) == 0);
    let keys = world
        .resource::<crate::persistence::Preferences>()
        .bindings
        .clone();
    let position = game
        .active
        .and_then(|e| world.get::<Transform>(e))
        .map_or(Vec3::ZERO, |t| t.translation);
    let shell = column(
        world,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(28),
            right: px(28),
            top: px(85),
            bottom: px(25),
            padding: UiRect::all(px(20)),
            column_gap: px(20),
            ..default()
        },
        INK,
    );
    let chart = column(
        world,
        shell,
        Node {
            width: px(730),
            min_width: px(730),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        Color::NONE,
    );
    text(world, chart, "ATLAS DES NEUF ARCHIPELS", 25.0, PAPER);
    button(
        world,
        chart,
        "Voyage rapide · îles et lieux remarquables",
        Action::Travel,
        false,
    );
    button(
        world,
        chart,
        "Atelier des circuits d'Aether",
        Action::Engineering,
        false,
    );
    text(
        world,
        chart,
        format!(
            "{} escales · {} îles · {} / 2 espèces observées · choisissez une destination",
            expedition.visited.len(),
            geography::islands().len() + 3,
            expedition.observed_species.len()
        ),
        13.0,
        MUTED,
    );
    let map = column(
        world,
        chart,
        Node {
            width: px(710),
            height: px(440),
            min_height: px(440),
            overflow: Overflow::clip(),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Color::srgb(0.028, 0.067, 0.10),
    );
    // The chart derives from the same routes and coordinates as the physical world.
    for route in geography::routes() {
        for segment in route.points.windows(2) {
            let a = point(segment[0]);
            let b = point(segment[1]);
            let delta = b - a;
            let middle = (a + b) * 0.5;
            world.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(middle.x - delta.length() * 0.5),
                    top: px(middle.y),
                    width: px(delta.length()),
                    height: px(1.5),
                    ..default()
                },
                UiTransform::from_rotation(Rot2::radians(delta.y.atan2(delta.x))),
                BackgroundColor(Color::srgba(0.17, 0.48, 0.76, 0.7)),
                ChildOf(map),
            ));
        }
    }
    for island in geography::islands().iter().filter(|i| !i.capital) {
        let p = point(island.center);
        let visited = expedition.visited.contains(&island.id);
        world.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(p.x),
                top: px(p.y),
                width: px(3.0),
                height: px(3.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(if visited {
                ACCENT
            } else {
                Color::srgba(0.4, 0.58, 0.62, 0.35)
            }),
            ChildOf(map),
        ));
    }
    for (id, p) in std::iter::once((0, Vec3::ZERO)).chain(
        geography::islands()
            .iter()
            .filter(|i| i.capital)
            .map(|i| (i.id, i.center)),
    ) {
        let p2 = point(p);
        let selected = expedition.destination == id;
        let name = format!("{}", id / 100);
        let node = button(world, map, &name, Action::Destination(id), selected);
        if let Some(mut n) = world.get_mut::<Node>(node) {
            n.position_type = PositionType::Absolute;
            n.left = px(p2.x - 16.0);
            n.top = px(p2.y - 16.0);
            n.width = px(32);
            n.min_height = px(32);
            n.padding = UiRect::all(px(5));
            n.justify_content = JustifyContent::Center;
        }
    }
    let p = point(position);
    let marker = text(world, map, "+", 22.0, PAPER);
    world.entity_mut(marker).insert(Node {
        position_type: PositionType::Absolute,
        left: px(p.x),
        top: px(p.y),
        ..default()
    });
    let legend = column(
        world,
        chart,
        Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(6),
            row_gap: px(5),
            ..default()
        },
        Color::NONE,
    );
    for island in geography::islands().iter().filter(|i| i.capital) {
        let name = format!("{}. {}", island.id / 100, island.biome.label());
        let b = button(
            world,
            legend,
            &name,
            Action::Destination(island.id),
            expedition.destination == island.id,
        );
        if let Some(mut n) = world.get_mut::<Node>(b) {
            n.width = px(230);
            n.min_height = px(35);
            n.padding = UiRect::all(px(5));
        }
    }
    let nearby = column(
        world,
        chart,
        Node {
            column_gap: px(6),
            ..default()
        },
        Color::NONE,
    );
    for id in 0..3 {
        button(
            world,
            nearby,
            &geography::port_name(id),
            Action::Destination(id),
            expedition.destination == id,
        );
    }
    text(
        world,
        chart,
        "+ Votre position  ·  0 Le Chantier  ·  Bleu : routes de courant\nL'altitude du quai est indiquée pour la destination choisie.",
        12.0,
        MUTED,
    );
    let panel = column(
        world,
        shell,
        Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(7),
            ..default()
        },
        Color::NONE,
    );
    let target = geography::dock(expedition.destination).unwrap_or(Vec3::ZERO);
    text(
        world,
        panel,
        geography::port_name(expedition.destination),
        25.0,
        ACCENT,
    );
    text(
        world,
        panel,
        format!(
            "À {:0.2} km · quai à {:0.0} m d'altitude\nLes lignes bleues indiquent les courants rapides.\nLes portails relient les escales déjà découvertes.",
            position.distance(target) / 1000.0,
            target.y
        ),
        13.0,
        PAPER,
    );
    button(
        world,
        panel,
        &format!(
            "Reprendre le voyage    {}",
            keys.label(crate::bindings::Control::Atlas)
        ),
        Action::Atlas,
        true,
    );
    text(
        world,
        panel,
        format!(
            "SOUTE · {} / 400 · {:.0} kg  |  {} crédits",
            expedition.cargo.iter().sum::<u32>(),
            expedition.cargo_mass(),
            expedition.credits
        ),
        16.0,
        ACCENT,
    );
    let biome = geography::island(checkpoint).map_or(geography::Biome::Dawn, |i| i.biome);
    for goods in Goods::ALL {
        let row = column(
            world,
            panel,
            Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: px(4),
                ..default()
            },
            Color::NONE,
        );
        text(
            world,
            row,
            format!("{}  ×{}", goods.label(), expedition.cargo[goods as usize]),
            13.0,
            PAPER,
        );
        if docked {
            button(
                world,
                row,
                &format!("Acheter {}", goods.price(biome)),
                Action::Trade(goods, true),
                false,
            );
            button(
                world,
                row,
                &format!("Vendre {}", goods.price(biome) * 3 / 4),
                Action::Trade(goods, false),
                false,
            );
        }
    }
    if docked && geography::island(checkpoint).is_some_and(|i| i.capital) {
        text(
            world,
            panel,
            format!(
                "COMPTOIR · {}\nContrat : 8 × {}",
                geography::port_name(checkpoint),
                contract_goods(biome).label()
            ),
            14.0,
            PAPER,
        );
        button(
            world,
            panel,
            if expedition.contracts.contains(&checkpoint) {
                "Contrat accompli"
            } else {
                "Livrer le contrat"
            },
            Action::Deliver,
            false,
        );
    } else if !docked {
        text(
            world,
            panel,
            format!(
                "Amarrez-vous pour commercer.\n{} : récolter cristaux et eau des bassins, ou observer la faune.",
                keys.label(crate::bindings::Control::Interact)
            ),
            13.0,
            MUTED,
        );
    }
    button(
        world,
        panel,
        "Raffiner 1 cristal pour 240 Aether",
        Action::Refine,
        false,
    );
    if docked && needs_motor {
        button(
            world,
            panel,
            "Motoriser : 4 bois + 4 fers + 2 cristaux",
            Action::MotorRefit,
            false,
        );
    }
    text(world, panel, notice, 13.0, PAPER);
}
