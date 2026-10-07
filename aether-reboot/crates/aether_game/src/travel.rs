//! Exploration shortcuts derive from the geography actually loaded by the game.
use crate::{
    app::Phase,
    controls::Action,
    interface::{button, column, text},
    session::GameSession,
};
use aether_core::{terrain, world as geography, world_geometry};
use aether_sim::Vessel;
use aether_view::widgets::{ACCENT, INK, MUTED, PAPER};
use avian3d::prelude::*;
use bevy::prelude::*;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Category {
    #[default]
    Highlights,
    Islands,
    Underground,
    Currents,
}
impl Category {
    const ALL: [Self; 4] = [
        Self::Highlights,
        Self::Islands,
        Self::Underground,
        Self::Currents,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Highlights => "À découvrir",
            Self::Islands => "Toutes les îles",
            Self::Underground => "Souterrains",
            Self::Currents => "Courants aériens",
        }
    }
}
#[derive(Resource, Default)]
pub struct TravelMenu {
    pub category: Category,
    pub page: usize,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Island,
    Landmark,
    Vault,
    Current,
}
pub struct Destination {
    pub id: u32,
    pub label: String,
    pub description: String,
    pub island: u32,
    pub position: Vec3,
    pub rotation: Quat,
    kind: Kind,
    featured: bool,
    underground: bool,
}
const PAGE_SIZE: usize = 12;
pub fn destinations() -> &'static [Destination] {
    static DATA: LazyLock<Vec<Destination>> = LazyLock::new(|| {
        let mut result = Vec::new();
        for id in (0..3).chain(geography::islands().iter().map(|i| i.id)) {
            let island = geography::island(id);
            let rotation = geography::dock_rotation(id);
            result.push(Destination {
                id,
                label: geography::port_name(id),
                description: if island.is_none_or(|i| i.capital) {
                    "Port, architecture et comptoir · arrivez devant le quai".into()
                } else {
                    format!(
                        "Îlot {} · {} · approche du quai",
                        id % 100,
                        island.unwrap().biome.label()
                    )
                },
                island: id,
                position: geography::dock(id).unwrap() + rotation * Vec3::new(0.0, 10.0, 45.0),
                rotation,
                kind: Kind::Island,
                featured: island.is_none_or(|i| i.capital),
                underground: island.is_some_and(|i| {
                    matches!(
                        i.biome,
                        geography::Biome::Hollow | geography::Biome::Underforge
                    )
                }),
            });
        }
        result.push(Destination {
            id: 30_000,
            label: "Observatoire ancien".into(),
            description: "Sanctuaire du premier archipel · architecture et ancrages de harpon"
                .into(),
            island: geography::nearest_port(terrain::SANCTUARY).0,
            position: terrain::SANCTUARY + Vec3::new(0.0, 15.0, 45.0),
            rotation: Quat::IDENTITY,
            kind: Kind::Landmark,
            featured: true,
            underground: false,
        });
        let mut extra = 10_000;
        for island in geography::islands()
            .iter()
            .filter(|i| i.capital || i.id == 123)
        {
            let marks = world_geometry::landmarks_for(island.asset_key());
            let underground = matches!(
                island.biome,
                geography::Biome::Hollow | geography::Biome::Underforge
            );
            let mut add = |name: &str, description: &str, local: Vec3| {
                let target = island.transform_point(local);
                let position =
                    target + Quat::from_rotation_y(island.yaw) * Vec3::new(0.0, 30.0, 35.0);
                result.push(Destination {
                    id: extra,
                    label: format!("{} · {}", name, island.biome.label()),
                    description: description.into(),
                    island: island.id,
                    position,
                    rotation: Quat::from_rotation_y(island.yaw),
                    kind: Kind::Landmark,
                    featured: true,
                    underground,
                });
                extra += 1;
            };
            if let Some(p) = marks.portal {
                add(
                    "Portail",
                    "Arche de passage · choisissez une escale découverte dans l'atlas",
                    p,
                );
            }
            if let Some(p) = marks.crystals.first() {
                add(
                    "Cristaux",
                    "Gisement récoltable · approchez puis interagissez",
                    *p,
                );
            }
            if let Some(p) = marks.pools.first() {
                add("Bassins", "Eau récoltable et terrasses", p.position);
            }
            if let Some(p) = marks.waterfalls.first() {
                add("Cascades", "Vue sur les chutes d'eau", p.position);
            }
            if underground {
                add(
                    "Faune des cavernes",
                    "Gardiens et planeurs · approchez puis observez",
                    Vec3::new(-25.0, 5.0, 40.0),
                );
            }
        }
        for vault in geography::vaults() {
            for (name, local) in [
                ("Grande cavité", Vec3::new(450.0, 180.0, 0.0)),
                ("Puits d'accès", Vec3::new(700.0, 320.0, 0.0)),
            ] {
                result.push(Destination {
                    id: extra,
                    label: format!("{} · {}", name, geography::port_name(vault.id)),
                    description: "Intérieur navigable · roche, lumière et courant ascendant".into(),
                    island: vault.id,
                    position: vault.transform_point(local),
                    rotation: Quat::from_rotation_y(vault.yaw),
                    kind: Kind::Vault,
                    featured: true,
                    underground: true,
                });
                extra += 1;
            }
        }
        for (index, route) in geography::routes().iter().enumerate() {
            let middle = route.points.len() / 2;
            let position = route.points[middle];
            let direction = route.points[(middle + 1).min(route.points.len() - 1)]
                - route.points[middle.saturating_sub(1)];
            result.push(Destination {
                id: 20_000 + index as u32,
                label: match index {
                    12 => "Spirale des Tempêtes".into(),
                    13 => "Anneau de l'Aube".into(),
                    _ => format!("Courant aérien {}", index + 1),
                },
                description: format!(
                    "{} m/s · trajet {} m · vent et navigation",
                    route.speed,
                    route
                        .points
                        .windows(2)
                        .map(|p| p[0].distance(p[1]))
                        .sum::<f32>()
                        .round()
                ),
                island: geography::nearest_port(position).0,
                position,
                rotation: Quat::from_rotation_y((-direction.x).atan2(-direction.z)),
                kind: Kind::Current,
                featured: true,
                underground: position.y < -600.0,
            });
        }
        result
    });
    &DATA
}
fn list(category: Category) -> Vec<&'static Destination> {
    destinations()
        .iter()
        .filter(|d| match category {
            Category::Highlights => d.featured,
            Category::Islands => d.kind == Kind::Island,
            Category::Underground => d.underground,
            Category::Currents => d.kind == Kind::Current,
        })
        .collect()
}
pub fn page(world: &mut World, next: bool) {
    let mut menu = world.resource_mut::<TravelMenu>();
    let pages = list(menu.category).len().div_ceil(PAGE_SIZE).max(1);
    menu.page = if next {
        (menu.page + 1).min(pages - 1)
    } else {
        menu.page.saturating_sub(1)
    };
}
pub fn build(world: &mut World, root: Entity) {
    let menu = world.resource::<TravelMenu>();
    let category = menu.category;
    let entries = list(category);
    let pages = entries.len().div_ceil(PAGE_SIZE).max(1);
    let page = menu.page.min(pages - 1);
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
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        INK,
    );
    text(world, shell, "Où souhaitez-vous explorer ?", 28.0, PAPER);
    text(
        world,
        shell,
        "Voyage gratuit, y compris vers les lieux encore inconnus. Votre navire, votre réserve et votre soute sont conservés.",
        14.0,
        MUTED,
    );
    let tabs = column(
        world,
        shell,
        Node {
            column_gap: px(8),
            ..default()
        },
        Color::NONE,
    );
    for tab in Category::ALL {
        button(
            world,
            tabs,
            tab.label(),
            Action::TravelTab(tab),
            tab == category,
        );
    }
    text(
        world,
        shell,
        format!(
            "{} lieux · page {} / {} · choisissez un lieu pour vous y rendre",
            entries.len(),
            page + 1,
            pages
        ),
        14.0,
        ACCENT,
    );
    let cards = column(
        world,
        shell,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            flex_grow: 1.0,
            ..default()
        },
        Color::NONE,
    );
    for group in entries
        .iter()
        .skip(page * PAGE_SIZE)
        .take(PAGE_SIZE)
        .collect::<Vec<_>>()
        .chunks(3)
    {
        let row = column(
            world,
            cards,
            Node {
                column_gap: px(12),
                ..default()
            },
            Color::NONE,
        );
        for d in group {
            let card = column(
                world,
                row,
                Node {
                    width: percent(33.333),
                    padding: UiRect::all(px(10)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(7),
                    min_height: px(110),
                    ..default()
                },
                Color::srgb(0.06, 0.085, 0.12),
            );
            button(world, card, &d.label, Action::FastTravel(d.id), false);
            text(world, card, &d.description, 12.0, MUTED);
        }
    }
    let notice = world.resource::<GameSession>().notice.clone();
    text(world, shell, notice, 12.0, MUTED);
    let footer = column(
        world,
        shell,
        Node {
            column_gap: px(10),
            ..default()
        },
        Color::NONE,
    );
    if page > 0 {
        button(
            world,
            footer,
            "Page précédente",
            Action::TravelPage(false),
            false,
        );
    }
    if page + 1 < pages {
        button(
            world,
            footer,
            "Page suivante",
            Action::TravelPage(true),
            false,
        );
    }
    button(
        world,
        footer,
        "Reprendre le voyage · Échap",
        Action::Resume,
        true,
    );
}
fn radius(body: &aether_core::Body) -> f32 {
    body.grid()
        .iter()
        .map(|(cell, _)| cell.center().length() + aether_core::CELL_SIZE)
        .chain(
            body.parts()
                .iter()
                .map(|p| p.center().length() + p.kind.size().length() * 0.5),
        )
        .fold(4.0, f32::max)
        + 1.5
}
fn clear(position: Vec3, radius: f32) -> bool {
    world_geometry::corridor_clear(position, position, radius)
        && terrain::obstacles().iter().all(|p| {
            let mut inflated = *p;
            inflated.size += Vec3::splat(radius * 2.0);
            !terrain::segment_intersects(position, position, &inflated)
        })
}
#[cfg(test)]
fn arrival(d: &Destination, radius: f32) -> Option<Vec3> {
    arrival_filtered(d, radius, |_| true)
}
fn arrival_filtered(
    d: &Destination,
    radius: f32,
    mut available: impl FnMut(Vec3) -> bool,
) -> Option<Vec3> {
    for lift in [0.0, 20.0, 50.0, 100.0] {
        for distance in [0.0, 12.0, 24.0, 48.0, 96.0, 160.0] {
            for angle in 0..8 {
                let a = angle as f32 * std::f32::consts::FRAC_PI_4;
                let p = d.position + Vec3::new(a.cos() * distance, lift, a.sin() * distance);
                if clear(p, radius) && available(p) {
                    return Some(p);
                }
                if distance == 0.0 {
                    break;
                }
            }
        }
    }
    None
}
fn relocate(world: &mut World, id: u32) -> Result<String, String> {
    let d = destinations()
        .iter()
        .find(|d| d.id == id)
        .ok_or("Destination inconnue.")?;
    let active = world
        .resource::<GameSession>()
        .active
        .ok_or("Aucun navire disponible.")?;
    let vessel = world.get::<Vessel>(active).ok_or("Navire indisponible.")?;
    let envelope = radius(&vessel.body);
    let traffic_radius = aether_core::traffic::collisions()
        .iter()
        .map(|p| p.center.length() + p.size.length() * 0.5)
        .fold(0.0, f32::max);
    let occupied: Vec<_> = world
        .query::<(
            Entity,
            &Position,
            Option<&Vessel>,
            Option<&aether_sim::traffic::Courier>,
        )>()
        .iter(world)
        .filter(|(e, _, _, _)| *e != active)
        .filter_map(|(_, p, v, c)| {
            v.map(|v| (p.0, radius(&v.body))).or_else(|| {
                c.map(|c| {
                    (
                        p.0,
                        c.0.sailing_body().map_or(traffic_radius, radius) * c.0.scale(),
                    )
                })
            })
        })
        .collect();
    let position = arrival_filtered(d, envelope, |p| {
        occupied
            .iter()
            .all(|(other, r)| p.distance(*other) > envelope + r + 2.0)
    })
    .ok_or("Votre navire est trop volumineux pour cette arrivée. Choisissez un autre lieu.")?;
    // Validate everything before releasing the cable or removing the walker.
    crate::controls::release_tether(world);
    if let Some(walker) = world.resource_mut::<GameSession>().walker.take() {
        world.despawn(walker);
    }
    aether_sim::streaming::ensure(world, d.island);
    world.entity_mut(active).insert((
        aether_sim::teleport_pose(position, d.rotation),
        RigidBody::Dynamic,
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
        ConstantForce::default(),
        ConstantTorque::default(),
        aether_sim::PilotIntent::default(),
    ));
    let mut vessel = world.get_mut::<Vessel>(active).unwrap();
    vessel.docked = false;
    vessel.target_altitude = position.y;
    if d.kind == Kind::Island {
        let mut game = world.resource_mut::<GameSession>();
        game.expedition.destination = d.island;
        game.expedition.visited.insert(d.island);
    }
    world.resource_mut::<crate::camera::CameraRig>().snap = true;
    Ok(format!(
        "Voyage rapide : {}. Vous êtes au poste de pilotage.",
        d.label
    ))
}
pub fn go(world: &mut World, id: u32) {
    match relocate(world, id) {
        Ok(message) => {
            world.resource_mut::<GameSession>().notice = message;
            world.resource_mut::<NextState<Phase>>().set(Phase::Playing);
            crate::session::request_save(world);
        }
        Err(message) => {
            world.resource_mut::<GameSession>().notice = message;
            crate::interface::rebuild(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_covers_every_real_island_and_all_routes_with_unique_ids() {
        let ids: std::collections::BTreeSet<_> = destinations().iter().map(|d| d.id).collect();
        assert_eq!(ids.len(), destinations().len());
        assert_eq!(
            list(Category::Islands).len(),
            geography::islands().len() + 3
        );
        for id in (0..3).chain(geography::islands().iter().map(|i| i.id)) {
            assert!(ids.contains(&id));
        }
        assert_eq!(list(Category::Currents).len(), geography::routes().len());
        assert_eq!(
            destinations()
                .iter()
                .filter(|d| d.kind == Kind::Vault)
                .count(),
            4
        );
    }
    #[test]
    fn every_destination_has_a_clear_arrival_for_the_actual_starter() {
        let radius = radius(&aether_core::fixtures::explorer());
        for d in destinations() {
            let p = arrival(d, radius).unwrap_or_else(|| panic!("blocked {}", d.label));
            assert!(p.is_finite() && clear(p, radius));
        }
    }
    #[test]
    fn travel_preserves_resources_and_resets_motion_and_walking() {
        let mut world = World::new();
        world.insert_resource(GameSession::default());
        world.insert_resource(crate::camera::CameraRig::default());
        let body = aether_core::fixtures::explorer();
        let properties = body.mass_properties();
        let active = world
            .spawn((
                Vessel {
                    id: aether_core::BodyId(1),
                    body,
                    properties,
                    payload: 37.0,
                    fuel: 123.0,
                    trim: 0.4,
                    docked: true,
                    target_altitude: 12.0,
                },
                LinearVelocity(Vec3::splat(90.0)),
                AngularVelocity(Vec3::Y * 4.0),
            ))
            .id();
        let walker = world.spawn_empty().id();
        world.resource_mut::<GameSession>().active = Some(active);
        world.resource_mut::<GameSession>().walker = Some(walker);
        let before = serde_json::to_string(&world.resource::<GameSession>().expedition).unwrap();
        assert!(relocate(&mut world, u32::MAX).is_err());
        assert!(world.get_entity(walker).is_ok());
        assert_eq!(
            world.get::<LinearVelocity>(active).unwrap().0,
            Vec3::splat(90.0)
        );
        let current = list(Category::Currents)[0].id;
        relocate(&mut world, current).unwrap();
        let vessel = world.get::<Vessel>(active).unwrap();
        assert_eq!(
            (vessel.fuel, vessel.payload, vessel.trim),
            (123.0, 37.0, 0.4)
        );
        assert!(!vessel.docked);
        assert_eq!(world.get::<LinearVelocity>(active).unwrap().0, Vec3::ZERO);
        assert_eq!(world.get::<AngularVelocity>(active).unwrap().0, Vec3::ZERO);
        assert!(world.get_entity(walker).is_err());
        assert_eq!(
            serde_json::to_string(&world.resource::<GameSession>().expedition).unwrap(),
            before
        );
        assert!(world.resource::<crate::camera::CameraRig>().snap);
    }
}
