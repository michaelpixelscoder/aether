use crate::{
    camera::MainCamera,
    controls::notice,
    session::{self, GameSession},
};
use aether_core::{
    Block, Body, BodyId, Cell, Part, PartId, PartKind,
    edit::{Edit, History},
    picking::{self, Hit},
    save::Session,
};
use aether_sim::{self as sim, Vessel};
use aether_view::BodyVisual;
use bevy::prelude::*;
use std::{collections::BTreeSet, sync::Arc};

#[derive(Resource)]
pub struct Editor {
    pub block: Block,
    pub part: Option<PartKind>,
    pub remove_part: bool,
    pub quarter_turn: u8,
    pub hit: Option<Hit>,
    pub history: History,
    pub fleet_undo: Option<Session>,
    pub fleet_redo: Option<Session>,
    pub preview_error: Option<String>,
    pub circuit_panel: bool,
    pub circuit_pick: Option<PartId>,
    preview_key: Option<(u64, Cell, Cell, String, bool)>,
    press: Option<Vec2>,
    dragged: bool,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            block: Block::Wood,
            part: None,
            remove_part: false,
            quarter_turn: 0,
            hit: None,
            history: History::default(),
            fleet_undo: None,
            fleet_redo: None,
            preview_error: None,
            circuit_panel: false,
            circuit_pick: None,
            preview_key: None,
            press: None,
            dragged: false,
        }
    }
}
impl Editor {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn selection(&self) -> String {
        if self.remove_part {
            "Retirer un composant".into()
        } else if let Some(p) = self.part {
            format!("{} · {}°", p.label(), self.quarter_turn as u16 * 90)
        } else {
            self.block.label().into()
        }
    }
}
pub fn pick(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    vessels: Query<(&Vessel, &GlobalTransform)>,
    ui: Query<&Interaction>,
    game: Res<GameSession>,
    mut editor: ResMut<Editor>,
    mut gizmos: Gizmos,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    editor.hit = None;
    if ui.iter().any(|i| *i != Interaction::None) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };
    let Some(entity) = game.active else {
        return;
    };
    let Ok((vessel, transform)) = vessels.get(entity) else {
        return;
    };
    let inverse = transform.affine().inverse();
    let origin = inverse.transform_point3(ray.origin);
    let direction = inverse.transform_vector3(*ray.direction);
    if let Some(hit) = picking::raycast(vessel.body.grid(), origin, direction, 200.0) {
        let remove = keys.pressed(KeyCode::ShiftLeft)
            || keys.pressed(KeyCode::ShiftRight)
            || buttons.pressed(MouseButton::Middle);
        let key = (
            vessel.body.revision(),
            hit.cell,
            hit.normal,
            editor.selection(),
            remove,
        );
        let proposed = proposal(&editor, &vessel.body, hit, remove);
        if editor.preview_key.as_ref() != Some(&key) {
            editor.preview_error = proposed
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|e| {
                    aether_core::edit::preview(&vessel.body, e).map_err(|e| e.to_string())
                })
                .err();
            editor.preview_key = Some(key);
        }
        let cells = match &proposed {
            Ok(Edit::Place(part)) => part.footprint(),
            Ok(Edit::Set(cell, _)) => vec![*cell],
            Ok(Edit::RemovePart(id)) => vessel
                .body
                .parts()
                .iter()
                .find(|p| p.id == *id)
                .map_or(vec![hit.cell], |p| p.footprint()),
            Ok(Edit::Circuit(_)) => vec![],
            Err(_) => vec![hit.cell],
        };
        let color = if editor.preview_error.is_some() {
            Color::srgb(1.0, 0.24, 0.25)
        } else if remove || editor.remove_part {
            Color::srgb(1.0, 0.67, 0.25)
        } else {
            Color::srgb(0.72, 0.91, 0.62)
        };
        for cell in cells {
            gizmos.cube(
                Transform::from_translation(transform.transform_point(cell.center()))
                    .with_rotation(transform.rotation())
                    .with_scale(Vec3::splat(0.51)),
                color,
            );
        }
        editor.hit = Some(hit);
    }
}
// Imported IDs may be near the format limit. Reserve a free positive ID rather
// than incrementing the largest; n used IDs leave a hole among 1..=n+1.
fn reserve_id(used: &mut BTreeSet<u64>) -> u64 {
    (1..=used.len() as u64 + 1)
        .find(|id| used.insert(*id))
        .expect("a finite set has a free ID below its cardinality plus two")
}
fn proposal(editor: &Editor, body: &Body, hit: Hit, remove: bool) -> Result<Edit, String> {
    Ok(if editor.remove_part {
        Edit::RemovePart(
            body.parts()
                .iter()
                .find(|p| p.cell == hit.cell)
                .ok_or("Visez la cellule qui porte le composant.")?
                .id,
        )
    } else if let Some(kind) = editor.part {
        let next = reserve_id(&mut body.parts().iter().map(|p| p.id.0).collect());
        Edit::Place(Part {
            id: PartId(next),
            kind,
            cell: hit.cell,
            quarter_turn: editor.quarter_turn,
        })
    } else if remove {
        Edit::Set(hit.cell, Block::Air)
    } else {
        Edit::Set(hit.cell.offset(hit.normal), editor.block)
    })
}
pub fn apply(world: &mut World) {
    if world.resource::<crate::persistence::Storage>().pending {
        return;
    }
    let Some(window) = world.query::<&Window>().iter(world).next() else {
        return;
    };
    let cursor = window.cursor_position();
    let focused = window.focused;
    if !focused {
        world.resource_mut::<Editor>().press = None;
        return;
    }
    let buttons = world.resource::<ButtonInput<MouseButton>>();
    let pressed = buttons.just_pressed(MouseButton::Left);
    let released = buttons.just_released(MouseButton::Left);
    let middle = buttons.just_pressed(MouseButton::Middle);
    let right = buttons.pressed(MouseButton::Right);
    let keys = world.resource::<ButtonInput<KeyCode>>();
    let remove = [KeyCode::ShiftLeft, KeyCode::ShiftRight]
        .into_iter()
        .any(|key| keys.pressed(key) || keys.just_released(key))
        || middle;
    {
        let mut editor = world.resource_mut::<Editor>();
        if pressed {
            editor.press = cursor;
            editor.dragged = false;
        }
        if let (Some(start), Some(now)) = (editor.press, cursor)
            && (start.distance(now) > 5.0 || right)
        {
            editor.dragged = true;
        }
        if !released && !middle {
            return;
        }
        let can_edit = (!editor.dragged && editor.press.take().is_some()) || middle;
        if !can_edit {
            return;
        }
    }
    let Some(entity) = world.resource::<GameSession>().active else {
        return;
    };
    let Some(hit) = world.resource::<Editor>().hit else {
        return;
    };
    let Some(vessel) = world.get::<Vessel>(entity) else {
        return;
    };
    if !vessel.docked {
        return;
    }
    let editor = world.resource::<Editor>();
    let edit = match proposal(editor, &vessel.body, hit, remove) {
        Ok(edit) => edit,
        Err(error) => {
            notice(world, error);
            return;
        }
    };
    execute(world, edit);
}
pub fn execute(world: &mut World, edit: Edit) {
    let Some(entity) = world.resource::<GameSession>().active else {
        return;
    };
    if !world.get::<Vessel>(entity).is_some_and(|v| v.docked) {
        return;
    }
    let mut result = Ok(vec![]);
    world.resource_scope(|world, mut editor: Mut<Editor>| {
        if let Some(mut vessel) = world.get_mut::<Vessel>(entity) {
            result = editor.history.apply(&mut vessel.body, edit);
        }
    });
    match result {
        Ok(dirty) => {
            world.resource_mut::<Editor>().fleet_undo = None;
            world.resource_mut::<Editor>().fleet_redo = None;
            refresh(world, entity, dirty);
            notice(world, "Construction mise à jour. Ctrl+Z pour annuler.");
        }
        Err(error) => notice(world, error.to_string()),
    }
}
pub fn refresh(world: &mut World, entity: Entity, mut dirty: Vec<Cell>) {
    let Some(body) = world.get::<Vessel>(entity).map(|v| v.body.clone()) else {
        return;
    };
    if body.circuit_design().is_none() {
        let reserve = world
            .get::<sim::aether::AetherCircuit>(entity)
            .map(sim::aether::AetherCircuit::conservative_common_fuel);
        world
            .entity_mut(entity)
            .remove::<sim::aether::AetherCircuit>();
        if let Some(reserve) = reserve {
            world.get_mut::<Vessel>(entity).expect("active vessel").fuel = reserve;
        }
    } else if let Some(mut circuit) = world.get_mut::<sim::aether::AetherCircuit>(entity) {
        circuit
            .reconcile_body(&body)
            .expect("validated construction circuit");
    } else {
        let fuel = world
            .get::<Vessel>(entity)
            .expect("active vessel")
            .fuel
            .min(body.fuel_capacity());
        let circuit = sim::aether::AetherCircuit::new(
            &body,
            body.circuit_design().expect("circuit design").clone(),
            fuel,
        )
        .expect("validated construction circuit");
        world.entity_mut(entity).insert(circuit);
    }
    let reserve = world
        .get::<sim::aether::AetherCircuit>(entity)
        .map(|c| c.remaining_milli() as f32 / 1000.0);
    let Some(mut vessel) = world.get_mut::<Vessel>(entity) else {
        return;
    };
    vessel.properties = vessel.body.mass_properties().with_payload(vessel.payload);
    vessel.fuel = reserve.unwrap_or_else(|| vessel.fuel.min(vessel.body.fuel_capacity()));
    let vessel = vessel.clone();
    world.resource_mut::<aether_view::MeshMetrics>().pending = 1;
    let children: Vec<_> = world
        .query::<(Entity, &ChildOf, &sim::HullCollider)>()
        .iter(world)
        .filter(|(_, parent, _)| parent.parent() == entity)
        .map(|(e, _, _)| e)
        .collect();
    for child in children {
        world.entity_mut(child).insert(sim::collider(&vessel.body));
    }
    let equipment: Vec<_> = world
        .query_filtered::<(Entity, &ChildOf), With<sim::EquipmentCollider>>()
        .iter(world)
        .filter(|(_, parent)| parent.parent() == entity)
        .map(|(e, _)| e)
        .collect();
    for child in equipment {
        world.despawn(child);
    }
    sim::spawn_equipment(&mut world.commands(), entity, &vessel.body);
    if dirty.is_empty() {
        dirty = vessel.body.grid().chunk_coords().collect();
        if let Some(old) = world.get::<BodyVisual>(entity) {
            dirty.extend(old.body.grid().chunk_coords());
        }
        dirty.sort();
        dirty.dedup();
    }
    world.entity_mut(entity).insert(BodyVisual {
        body: Arc::new(vessel.body.clone()),
        dirty,
    });
    sim::replace_geometry(&mut world.commands(), entity, &vessel);
    world.flush();
}
pub fn history(world: &mut World, redo: bool) {
    let fleet = if redo {
        world.resource_mut::<Editor>().fleet_redo.take()
    } else {
        world.resource_mut::<Editor>().fleet_undo.take()
    };
    if let Some(mut restore) = fleet {
        match session::snapshot(world) {
            Ok(current) => {
                if let Err(error) = restore.limit_reserve_from(&current) {
                    notice(world, error.to_string());
                    return;
                }
                session::replace(world, restore);
                if redo {
                    world.resource_mut::<Editor>().fleet_undo = Some(current);
                } else {
                    world.resource_mut::<Editor>().fleet_redo = Some(current);
                }
                notice(
                    world,
                    "Séparation annulée / rétablie. Quantité d'Aether conservée.",
                );
            }
            Err(error) => notice(world, error),
        }
        return;
    }
    let Some(entity) = world.resource::<GameSession>().active else {
        return;
    };
    let mut result = Ok(false);
    world.resource_scope(|world, mut editor: Mut<Editor>| {
        if let Some(mut vessel) = world.get_mut::<Vessel>(entity) {
            result = if redo {
                editor.history.redo(&mut vessel.body)
            } else {
                editor.history.undo(&mut vessel.body)
            };
        }
    });
    match result {
        Ok(true) => {
            refresh(world, entity, vec![]);
            notice(
                world,
                if redo {
                    "Édition rétablie."
                } else {
                    "Édition annulée."
                },
            );
        }
        Ok(false) => notice(world, "Historique vide."),
        Err(error) => notice(world, error.to_string()),
    }
}
pub fn split(world: &mut World) {
    let Some(entity) = world.resource::<GameSession>().active else {
        return;
    };
    let Some(vessel) = world.get::<Vessel>(entity) else {
        return;
    };
    if !vessel.docked {
        notice(world, "La séparation s'effectue au quai.");
        return;
    }
    let components = vessel.body.split();
    if components.len() < 2 {
        notice(world, "La coque est d'un seul tenant.");
        return;
    }
    let Ok(before) = session::snapshot(world) else {
        return;
    };
    if before.vessels.len() + components.len() - 1 > aether_core::MAX_BODIES {
        notice(
            world,
            "Limite de 32 corps atteinte : reconnectez les fragments.",
        );
        return;
    }
    let mut after = before.clone();
    let Some(index) = after.vessels.iter().position(|v| v.id == after.active) else {
        return;
    };
    let source = after.vessels.remove(index);
    let original = Body::from_blueprint(source.blueprint.clone()).expect("validated");
    let old_center = original.mass_properties().center;
    let total_capacity = original.fuel_capacity();
    let mut used_ids = before.vessels.iter().map(|v| v.id.0).collect();
    for (i, blueprint) in components.into_iter().enumerate() {
        let body = Body::from_blueprint(blueprint.clone()).expect("split retains valid parts");
        let properties = body.mass_properties();
        let mut fragment = source.clone();
        fragment.blueprint = blueprint;
        fragment.id = if i == 0 {
            source.id
        } else {
            BodyId(reserve_id(&mut used_ids))
        };
        fragment.velocity = source.velocity
            + source
                .angular_velocity
                .cross(source.rotation * (properties.center - old_center));
        fragment.fuel = if total_capacity > 0.0 {
            source.fuel * body.fuel_capacity() / total_capacity
        } else {
            0.0
        };
        if let Some(state) = &source.circuit {
            let circuit = state.for_fragment(&body).expect("validated split circuit");
            fragment.fuel = circuit
                .validated_network()
                .expect("validated fragment reserve")
                .amount() as f32
                / 1000.0;
            fragment.circuit = Some(circuit);
        }
        after.vessels.push(fragment);
    }
    after.tether = None;
    session::replace(world, after);
    world.resource_mut::<Editor>().fleet_undo = Some(before);
    notice(
        world,
        "Fragments séparés. Ctrl+Z pour réunir la construction.",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imported_high_part_id_still_allows_construction() {
        let mut blueprint = aether_core::fixtures::starter().blueprint();
        blueprint.parts[0].id = PartId(u64::MAX - 1024);
        let mut body = Body::from_blueprint(blueprint).unwrap();
        let editor = Editor {
            part: Some(PartKind::Lift),
            ..default()
        };
        let hit = Hit {
            cell: Cell(2, 0, 2),
            normal: Cell(0, 1, 0),
            distance: 1.0,
        };
        let edit = proposal(&editor, &body, hit, false).unwrap();
        History::default().apply(&mut body, edit).unwrap();
        assert_eq!(body.parts().len(), 7);
        assert!(body.parts().iter().any(|p| p.id == PartId(u64::MAX - 1024)));
    }

    #[test]
    fn imported_high_body_ids_allow_a_full_valid_fleet() {
        let mut saved = session::starter();
        saved.vessels[0].id = BodyId(u64::MAX - 1024);
        saved.active = saved.vessels[0].id;
        let mut used = saved.vessels.iter().map(|v| v.id.0).collect();
        while saved.vessels.len() < aether_core::MAX_BODIES {
            let mut fragment = saved.vessels[0].clone();
            fragment.id = BodyId(reserve_id(&mut used));
            saved.vessels.push(fragment);
        }
        assert_eq!(used.len(), aether_core::MAX_BODIES);
        assert!(saved.encode().is_ok());
        assert_eq!(saved.active, BodyId(u64::MAX - 1024));
    }
}
