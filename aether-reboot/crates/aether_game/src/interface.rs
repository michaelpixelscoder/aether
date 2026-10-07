use crate::{
    app::Phase,
    controls::{Action, Actions},
    editor::Editor,
    persistence::{Preferences, Storage},
    session::GameSession,
};
use aether_core::{Block, PartKind};
use aether_sim::{FlightTelemetry, Vessel};
use aether_view::widgets::{self, ACCENT, INK, MUTED, PAPER};
use bevy::prelude::*;

#[derive(Component)]
pub struct UiRoot;
#[derive(Component, Clone, Copy)]
pub struct UiAction(pub Action);
#[derive(Component)]
pub struct ButtonStyle(pub bool);
#[derive(Resource, Default)]
pub struct KeyboardUi {
    pub focused: Option<Entity>,
    pub active: bool,
    pub consumed: bool,
}
#[derive(Resource, Default)]
pub struct PhotoMode(pub bool);
pub fn photo(
    keys: Res<ButtonInput<KeyCode>>,
    capture: Res<crate::bindings::BindingCapture>,
    phase: Res<State<Phase>>,
    mut mode: ResMut<PhotoMode>,
    mut ui: Query<&mut Visibility, With<UiRoot>>,
) {
    let next_mode = if matches!(phase.get(), Phase::Settings | Phase::Travel) {
        false
    } else if capture.active.is_none() && keys.just_pressed(KeyCode::F8) {
        !mode.0
    } else {
        mode.0
    };
    if mode.0 != next_mode {
        mode.0 = next_mode;
        for mut visibility in &mut ui {
            *visibility = if mode.0 {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
    }
}
#[derive(Component)]
pub struct StatusText;
#[derive(Component)]
pub struct StatsText;
#[derive(Component)]
pub struct ObjectiveText;
#[derive(Component)]
pub struct SelectionText;
#[derive(Component)]
pub struct FuelBar;
#[derive(Component)]
pub struct SettingsText;
#[derive(Component)]
pub struct BindingMessage;

pub(crate) fn text(
    world: &mut World,
    parent: Entity,
    value: impl Into<String>,
    size: f32,
    color: Color,
) -> Entity {
    world
        .spawn((widgets::label(value, size, color), ChildOf(parent)))
        .id()
}
pub(crate) fn column(world: &mut World, parent: Entity, node: Node, background: Color) -> Entity {
    world
        .spawn((node, BackgroundColor(background), ChildOf(parent)))
        .id()
}
pub(crate) fn button(
    world: &mut World,
    parent: Entity,
    label: &str,
    action: Action,
    primary: bool,
) -> Entity {
    use crate::bindings::Control;
    let control = match action {
        Action::Edit => Some(Control::Edit),
        Action::Dock => Some(Control::Dock),
        Action::Tether => Some(Control::Tether),
        _ => None,
    };
    let resolved = control.map(|c| {
        format!(
            "{}    {}",
            label.split("    ").next().unwrap_or(label),
            world.resource::<Preferences>().bindings.label(c)
        )
    });
    let label = resolved.as_deref().unwrap_or(label);
    let entity = world
        .spawn((
            Button,
            UiAction(action),
            ButtonStyle(primary),
            Outline::new(px(2), px(2), Color::NONE),
            Node {
                min_height: px(35),
                padding: UiRect::axes(px(14), px(8)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            BackgroundColor(if primary {
                ACCENT
            } else {
                Color::srgb(0.10, 0.135, 0.18)
            }),
            ChildOf(parent),
        ))
        .id();
    text(
        world,
        entity,
        label,
        14.0,
        if primary { INK } else { PAPER },
    );
    entity
}
pub fn rebuild(world: &mut World) {
    if let Some(mut focus) = world.get_resource_mut::<KeyboardUi>() {
        *focus = default();
    }
    let old: Vec<_> = world
        .query_filtered::<Entity, With<UiRoot>>()
        .iter(world)
        .collect();
    for entity in old {
        world.despawn(entity);
    }
    let phase = *world.resource::<State<Phase>>().get();
    let prefs = world.resource::<Preferences>().clone();
    let root = world
        .spawn((
            UiRoot,
            if world.resource::<PhotoMode>().0 {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            },
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
            GlobalZIndex(20),
        ))
        .id();
    let header = column(
        world,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(28),
            right: px(28),
            top: px(22),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(16), px(7)),
            border_radius: BorderRadius::all(px(5)),
            ..default()
        },
        Color::srgba(0.035, 0.050, 0.075, 0.9),
    );
    text(world, header, "AETHER ISLES", 23.0, PAPER);
    let chapter = match phase {
        Phase::Boot | Phase::Loading => "PRÉPARATION DE L'ARCHIPEL",
        Phase::Error => "CHARGEMENT INTERROMPU",
        Phase::Menu => "01 / LES PREMIERS COURANTS",
        Phase::Editing => "ATELIER / CONSTRUCTION AU QUAI",
        Phase::Playing => "LES NEUF ARCHIPELS / EXPÉDITION",
        Phase::Paused => "TRAVERSÉE EN PAUSE",
        Phase::Settings => "PRÉFÉRENCES",
        Phase::Travel => "VOYAGE RAPIDE / EXPLORATION",
        Phase::Atlas => {
            if world.resource::<Editor>().circuit_panel {
                "ATELIER / DISTRIBUTION D'AETHER"
            } else {
                "ATLAS / ROUTES ET COMPTOIRS"
            }
        }
    };
    text(world, header, chapter, 12.0, MUTED);
    if phase == Phase::Atlas {
        crate::atlas::build(world, root);
        return;
    }
    if phase == Phase::Travel {
        crate::travel::build(world, root);
        return;
    }
    if matches!(phase, Phase::Boot | Phase::Loading | Phase::Error) {
        let panel = column(
            world,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: percent(10),
                top: percent(30),
                width: percent(80),
                max_width: px(800),
                ..widgets::panel()
            },
            INK,
        );
        text(
            world,
            panel,
            if phase == Phase::Error {
                "L'archipel n'a pas pu s'ouvrir."
            } else {
                "Préparation du voyage…"
            },
            34.0,
            PAPER,
        );
        if phase == Phase::Error {
            let message = world.resource::<crate::app::LoadingStatus>().error.clone();
            text(world, panel, message, 16.0, PAPER);
            button(world, panel, "Réessayer", Action::Retry, true);
        } else {
            text(
                world,
                panel,
                "Chargement des textures et des polices.",
                16.0,
                MUTED,
            );
        }
        return;
    }
    if matches!(phase, Phase::Menu | Phase::Paused | Phase::Settings) {
        let panel = column(
            world,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: percent(6),
                top: percent(20),
                width: px(410),
                max_width: percent(88),
                padding: UiRect::all(px(28)),
                row_gap: px(13),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            Color::srgba(0.035, 0.050, 0.075, 0.94),
        );
        world.entity_mut(panel).insert(Interaction::None);
        if phase == Phase::Menu {
            text(world, panel, "CONSTRUIRE. PRENDRE LE LARGE.", 11.0, ACCENT);
            text(world, panel, "Le ciel est\nà construire.", 47.0, PAPER);
            text(
                world,
                panel,
                "Neuf archipels, des cités dans le ciel.\nConstruisez, explorez et commercez.\nLe vent accélère votre expédition.",
                16.0,
                MUTED,
            );
            button(
                world,
                panel,
                "Commencer la traversée    >",
                Action::NewGame,
                true,
            );
            button(world, panel, "Reprendre la sauvegarde", Action::Load, false);
            button(world, panel, "Préférences", Action::Settings, false);
            text(world, panel, "SOURIS + CLAVIER  /  SOLO", 10.0, MUTED);
        } else if phase == Phase::Paused {
            text(world, panel, "À votre rythme.", 34.0, PAPER);
            button(world, panel, "Reprendre    >", Action::Pause, true);
            button(world, panel, "Sauvegarder", Action::Save, false);
            button(world, panel, "Charger la sauvegarde", Action::Load, false);
            button(world, panel, "Préférences", Action::Settings, false);
            button(world, panel, "Retour au menu", Action::Menu, false);
            button(world, panel, "Voyage rapide", Action::Travel, false);
            button(
                world,
                panel,
                "Rejouer les conseils",
                Action::Tutorial,
                false,
            );
        } else {
            text(world, panel, "Votre traversée.", 34.0, PAPER);
            button(
                world,
                panel,
                &format!(
                    "Sons : {}",
                    if prefs.volume > 0.0 {
                        "activés"
                    } else {
                        "coupés"
                    }
                ),
                Action::Volume,
                false,
            );
            button(
                world,
                panel,
                &format!(
                    "Mouvements de caméra réduits : {}",
                    if prefs.reduce_motion { "oui" } else { "non" }
                ),
                Action::Motion,
                false,
            );
            button(
                world,
                panel,
                &format!(
                    "Axe vertical inversé : {}",
                    if prefs.invert_y { "oui" } else { "non" }
                ),
                Action::Invert,
                false,
            );
            button(
                world,
                panel,
                &format!("Sensibilité souris : {:.1}", prefs.sensitivity),
                Action::Sensitivity,
                false,
            );
            button(world, panel, "Retour    >", Action::Pause, true);
            let keys = column(
                world,
                root,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(500),
                    right: px(24),
                    top: px(112),
                    bottom: px(76),
                    padding: UiRect::all(px(18)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6),
                    ..default()
                },
                INK,
            );
            text(world, keys, "Commandes", 28.0, PAPER);
            let message = world
                .resource::<crate::bindings::BindingCapture>()
                .message
                .clone();
            let message_entity = text(
                world,
                keys,
                if message.is_empty() {
                    "Choisissez une action, puis sa touche.".into()
                } else {
                    message
                },
                12.0,
                MUTED,
            );
            world.entity_mut(message_entity).insert((
                BindingMessage,
                Node {
                    height: px(34),
                    min_height: px(34),
                    ..default()
                },
            ));
            let swap_slot = column(
                world,
                keys,
                Node {
                    height: px(35),
                    min_height: px(35),
                    ..default()
                },
                Color::NONE,
            );
            button(
                world,
                swap_slot,
                "Échanger les touches (Entrée)",
                Action::SwapBinding,
                false,
            );
            let rows = column(
                world,
                keys,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(10),
                    ..default()
                },
                Color::NONE,
            );
            for controls in crate::bindings::Control::ALL.chunks(8) {
                let group = column(
                    world,
                    rows,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(5),
                        flex_grow: 1.0,
                        flex_basis: px(0),
                        min_width: px(0),
                        ..default()
                    },
                    Color::NONE,
                );
                for &control in controls {
                    let row = button(
                        world,
                        group,
                        &format!("{} : {}", control.label(), prefs.bindings.label(control)),
                        Action::Rebind(control),
                        false,
                    );
                    if let Some(mut node) = world.get_mut::<Node>(row) {
                        node.height = px(44);
                        node.min_height = px(44);
                        node.max_height = px(44);
                        node.padding = UiRect::axes(px(10), px(6));
                    }
                    let children = world
                        .get::<Children>(row)
                        .unwrap()
                        .iter()
                        .collect::<Vec<_>>();
                    for child in children {
                        world.get_mut::<TextFont>(child).unwrap().font_size =
                            bevy::text::FontSize::Px(12.0);
                    }
                }
            }
            button(
                world,
                keys,
                "Rétablir les touches",
                Action::ResetBindings,
                false,
            );
            text(
                world,
                keys,
                "Lettres, Maj, flèches, Espace ou Tab. Échap, Entrée, F3, F6, F8 et Ctrl+S/O/Z/Y sont réservés. F6 : boutons au clavier.",
                11.0,
                MUTED,
            );
            sync_bindings(world);
        }
    } else {
        let panel = column(
            world,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                top: px(76),
                width: px(266),
                padding: UiRect::all(px(16)),
                row_gap: px(7),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            Color::srgba(0.035, 0.055, 0.08, 0.94),
        );
        world.entity_mut(panel).insert(Interaction::None);
        if phase == Phase::Editing {
            text(world, panel, "LE CHANTIER", 12.0, ACCENT);
            let selection = text(world, panel, "Bois", 20.0, PAPER);
            world.entity_mut(selection).insert(SelectionText);
            let row = column(
                world,
                panel,
                Node {
                    column_gap: px(6),
                    ..default()
                },
                Color::NONE,
            );
            button(world, row, "Bois", Action::SelectBlock(Block::Wood), false);
            button(
                world,
                row,
                "Métal",
                Action::SelectBlock(Block::Metal),
                false,
            );
            button(
                world,
                row,
                "Verre",
                Action::SelectBlock(Block::Glass),
                false,
            );
            for kind in [
                PartKind::Sail,
                PartKind::Tank,
                PartKind::Lift,
                PartKind::Helm,
                PartKind::Harpoon,
                PartKind::Propeller,
                PartKind::GrandSail,
            ] {
                button(world, panel, kind.label(), Action::SelectPart(kind), false);
            }
            button(
                world,
                panel,
                "Retirer un composant",
                Action::RemovePart,
                false,
            );
            let row = column(
                world,
                panel,
                Node {
                    column_gap: px(6),
                    ..default()
                },
                Color::NONE,
            );
            button(world, row, "Annuler", Action::Undo, false);
            button(world, row, "Rétablir", Action::Redo, false);
            button(world, panel, "Séparer les fragments", Action::Split, false);
            button(
                world,
                panel,
                "Distribution d'Aether",
                Action::Engineering,
                false,
            );
            let row = column(
                world,
                panel,
                Node {
                    column_gap: px(6),
                    ..default()
                },
                Color::NONE,
            );
            button(world, row, "Exporter", Action::Export, false);
            button(world, row, "Importer", Action::Import, false);
            button(
                world,
                panel,
                "Terminer l'atelier    Tab",
                Action::Edit,
                true,
            );
            text(
                world,
                panel,
                format!(
                    "Clic : placer · Maj+clic : retirer\n1 2 3 : matériaux · {} : rotation\nClic droit : orbite · Molette : zoom\nF6 : boutons au clavier",
                    prefs.bindings.label(crate::bindings::Control::Rotate)
                ),
                11.0,
                MUTED,
            );
        } else {
            text(world, panel, "VOTRE CAP", 11.0, ACCENT);
            let objective = text(world, panel, "Préparez votre vaisseau.", 18.0, PAPER);
            world.entity_mut(objective).insert(ObjectiveText);
            text(
                world,
                panel,
                {
                    use crate::bindings::Control::*;
                    let k = |c| prefs.bindings.keys(c)[0].label();
                    format!(
                        "{}  Départ / amarrage\n{}  Atelier au quai\n{} {}  Gouvernail\n{}  Frein · {} {}  Voile\n{} {}  Monter / descendre\n{}  Harpon / libérer\n{}  Marcher / piloter\n{}  Secours au dernier quai\n{} / {}  Moteurs avant / arrière\n{}  Atlas · {}  Récolter / interagir",
                        k(Dock),
                        k(Edit),
                        k(Left),
                        k(Right),
                        k(Backward),
                        k(TrimLeft),
                        k(TrimRight),
                        k(Ascend),
                        k(Descend),
                        k(Tether),
                        k(Walk),
                        k(Recover),
                        k(Forward),
                        k(Reverse),
                        k(Atlas),
                        k(Interact)
                    )
                },
                12.0,
                MUTED,
            );
            button(world, panel, "Atelier    Tab", Action::Edit, false);
            button(world, panel, "Départ / amarrage    F", Action::Dock, true);
            button(world, panel, "Harpon    G", Action::Tether, false);
            button(
                world,
                panel,
                "Atlas et comptoirs    M",
                Action::Atlas,
                false,
            );
            button(world, panel, "Sauvegarder", Action::Save, false);
        }
        let info = column(
            world,
            root,
            Node {
                position_type: PositionType::Absolute,
                right: px(24),
                top: px(76),
                width: px(235),
                padding: UiRect::all(px(16)),
                row_gap: px(10),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            Color::srgba(0.035, 0.055, 0.08, 0.90),
        );
        world.entity_mut(info).insert(Interaction::None);
        text(world, info, "L'ALCYON  /  TÉLÉMÉTRIE", 11.0, ACCENT);
        let stats = text(world, info, "", 14.0, PAPER);
        world.entity_mut(stats).insert(StatsText);
        let track = column(
            world,
            info,
            Node {
                height: px(5),
                width: percent(100),
                ..default()
            },
            Color::srgb(0.16, 0.19, 0.25),
        );
        let fill = column(
            world,
            track,
            Node {
                height: percent(100),
                width: percent(100),
                ..default()
            },
            ACCENT,
        );
        world.entity_mut(fill).insert(FuelBar);
    }
    let footer = column(
        world,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(24),
            right: px(24),
            bottom: px(20),
            min_height: px(43),
            padding: UiRect::axes(px(16), px(12)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        Color::srgba(0.035, 0.050, 0.075, 0.94),
    );
    world.entity_mut(footer).insert(Interaction::None);
    let status = text(world, footer, "", 12.0, PAPER);
    world.entity_mut(status).insert(StatusText);
    text(world, footer, "ÉCHAP  Pause", 11.0, MUTED);
}
pub fn buttons(
    mut query: Query<
        (
            &Interaction,
            &UiAction,
            &ButtonStyle,
            &Children,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
    mut colors: Query<&mut TextColor>,
    mut actions: ResMut<Actions>,
) {
    for (interaction, action, style, children, mut background) in &mut query {
        let primary = style.0 || *interaction == Interaction::Pressed;
        for child in children.iter() {
            if let Ok(mut color) = colors.get_mut(child) {
                color.0 = if primary { INK } else { PAPER };
            }
        }
        match interaction {
            Interaction::Pressed => {
                background.0 = ACCENT;
                actions.0.push_back(action.0);
            }
            Interaction::Hovered => {
                background.0 = if style.0 {
                    Color::srgb(0.77, 0.65, 1.0)
                } else {
                    Color::srgb(0.26, 0.24, 0.39)
                }
            }
            Interaction::None => {
                background.0 = if style.0 {
                    ACCENT
                } else {
                    Color::srgb(0.10, 0.135, 0.18)
                }
            }
        }
    }
}
pub fn keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    phase: Res<State<Phase>>,
    capture: Res<crate::bindings::BindingCapture>,
    mut focus: ResMut<KeyboardUi>,
    mut actions: ResMut<Actions>,
    mut buttons: Query<(Entity, &UiAction, &UiGlobalTransform, &mut Outline)>,
    photo: Res<PhotoMode>,
) {
    focus.consumed = false;
    if photo.0 {
        focus.active = false;
        focus.focused = None;
        return;
    }
    if capture.active.is_some() {
        return;
    }
    if mouse.just_pressed(MouseButton::Left) {
        focus.active = false;
        focus.focused = None;
    }
    let menu = matches!(
        phase.get(),
        Phase::Menu | Phase::Paused | Phase::Settings | Phase::Error | Phase::Atlas | Phase::Travel
    );
    if keys.just_pressed(KeyCode::F6) {
        focus.active = !focus.active;
        focus.consumed = true;
    }
    if menu
        && (keys.just_pressed(KeyCode::Tab)
            || keys.just_pressed(KeyCode::ArrowDown)
            || keys.just_pressed(KeyCode::ArrowUp))
    {
        focus.active = true;
    }
    if focus.active {
        let mut order: Vec<_> = buttons
            .iter()
            .filter(|(_, a, _, _)| a.0 != Action::SwapBinding || capture.conflict.is_some())
            .map(|(e, a, t, _)| (e, a.0, t.affine().translation))
            .collect();
        order.sort_by(|a, b| {
            a.2.y
                .total_cmp(&b.2.y)
                .then_with(|| a.2.x.total_cmp(&b.2.x))
        });
        if !order.is_empty() {
            let previous = focus
                .focused
                .and_then(|e| order.iter().position(|b| b.0 == e));
            let backwards = keys.just_pressed(KeyCode::ArrowUp)
                || (keys.just_pressed(KeyCode::Tab) && keys.pressed(KeyCode::ShiftLeft));
            let forwards = keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::Tab);
            let index = if backwards {
                previous.map_or(order.len() - 1, |i| (i + order.len() - 1) % order.len())
            } else if forwards {
                previous.map_or(0, |i| (i + 1) % order.len())
            } else {
                previous.unwrap_or(0)
            };
            focus.focused = Some(order[index].0);
            if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
                actions.0.push_back(order[index].1);
                focus.consumed = true;
            }
        }
        if keys.just_pressed(KeyCode::Escape) && !menu {
            focus.active = false;
            focus.focused = None;
            focus.consumed = true;
        }
    }
    for (entity, _, _, mut outline) in &mut buttons {
        outline.color = if focus.active && focus.focused == Some(entity) {
            PAPER
        } else {
            Color::NONE
        };
    }
}

/// Update existing rows: rebuilding the root drops focus and causes a blank
/// layout frame. The conflict slot is reserved so its appearance cannot jump.
pub fn sync_bindings(world: &mut World) {
    let capture = world.resource::<crate::bindings::BindingCapture>();
    let message = if capture.message.is_empty() {
        "Choisissez une action, puis sa touche.".to_owned()
    } else {
        capture.message.clone()
    };
    let selected = capture.selected;
    let conflict = capture.conflict;
    let bindings = world.resource::<Preferences>().bindings.clone();
    for mut label in world
        .query_filtered::<&mut Text, With<BindingMessage>>()
        .iter_mut(world)
    {
        label.0.clone_from(&message);
    }
    let rows: Vec<_> = world
        .query::<(Entity, &UiAction, &Children)>()
        .iter(world)
        .map(|(entity, action, children)| (entity, action.0, children.iter().collect::<Vec<_>>()))
        .collect();
    for (entity, action, children) in rows {
        let (label, primary) = match action {
            Action::Rebind(control) => (
                format!("{} : {}", control.label(), bindings.label(control)),
                selected == Some(control),
            ),
            Action::SwapBinding => {
                if let Some(mut visibility) = world.get_mut::<Visibility>(entity) {
                    *visibility = if conflict.is_some() {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                } else {
                    world.entity_mut(entity).insert(if conflict.is_some() {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    });
                }
                (
                    conflict.map_or_else(
                        || "Échanger les touches (Entrée)".into(),
                        |(key, other)| {
                            format!(
                                "Échanger {} avec « {} » (Entrée)",
                                key.label(),
                                other.label()
                            )
                        },
                    ),
                    false,
                )
            }
            _ => continue,
        };
        world.get_mut::<ButtonStyle>(entity).unwrap().0 = primary;
        world.get_mut::<BackgroundColor>(entity).unwrap().0 = if primary {
            ACCENT
        } else {
            Color::srgb(0.10, 0.135, 0.18)
        };
        for child in children {
            if let Some(mut text) = world.get_mut::<Text>(child) {
                text.0.clone_from(&label);
            }
            if let Some(mut color) = world.get_mut::<TextColor>(child) {
                color.0 = if primary { INK } else { PAPER };
            }
        }
    }
}
pub fn update_hud(
    phase: Res<State<Phase>>,
    game: Res<GameSession>,
    editor: Res<Editor>,
    storage: Res<Storage>,
    prefs: Res<Preferences>,
    tutorial: Res<crate::tutorial::Tutorial>,
    vessels: Query<(
        &Vessel,
        &Transform,
        &avian3d::prelude::LinearVelocity,
        &FlightTelemetry,
    )>,
    mut texts: Query<(
        &mut Text,
        Option<&StatusText>,
        Option<&StatsText>,
        Option<&ObjectiveText>,
        Option<&SelectionText>,
    )>,
    mut fuel_bars: Query<&mut Node, With<FuelBar>>,
    tethers: Query<&aether_sim::Tether>,
    transforms: Query<&Transform>,
    residents: Query<(&aether_sim::fauna::Resident, &Transform)>,
) {
    let vessel = game.active.and_then(|e| vessels.get(e).ok());
    let interest = game
        .walker
        .or(game.active)
        .and_then(|e| transforms.get(e).ok())
        .and_then(|t| {
            let resources = aether_core::world::islands()
                .iter()
                .flat_map(|i| {
                    aether_core::expedition::resource_nodes(i).map(move |node| (i.id, node))
                })
                .filter(|(id, n)| !game.expedition.harvested.contains(&(id * 64 + n.slot)))
                .map(|(_, n)| {
                    (
                        n.goods.label(),
                        (t.translation.distance(n.position) - n.radius).max(0.0),
                        "Récolter",
                    )
                });
            let creatures = residents.iter().map(|(r, p)| {
                (
                    r.0.species.label(),
                    p.translation.distance(t.translation),
                    "Observer",
                )
            });
            resources
                .chain(creatures)
                .filter(|(_, d, _)| *d < 45.0)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(name, d, verb)| {
                    format!(
                        "{name} · {d:.0} m\n{verb} à proximité avec {}",
                        prefs.bindings.label(crate::bindings::Control::Interact)
                    )
                })
        });
    for (mut text, status, stats, objective, selection) in &mut texts {
        let value = if status.is_some() {
            Some(if storage.pending {
                "Opération de stockage en cours…".into()
            } else if *phase.get() == Phase::Editing
                && editor.hit.is_some()
                && editor.preview_error.is_some()
            {
                editor.preview_error.clone().unwrap_or_default()
            } else {
                game.notice.clone()
            })
        } else if stats.is_some() {
            vessel.map(|(v,t,velocity,telemetry)|{
                let target=aether_core::world::dock(game.expedition.destination).unwrap_or(aether_sim::ISLANDS[1].dock);
                let to=target-t.translation;
                let yaw=t.rotation.to_euler(EulerRot::YXZ).0;
                let angle=((-to.x).atan2(-to.z)-yaw+std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)-std::f32::consts::PI;
                let cap=if angle.abs()<0.15 {"devant"}else if angle>0.0{"à gauche"}else{"à droite"};
                let cable=tethers.iter().next().map_or("Câble libre".to_string(),|c|{
                    let distance=transforms.get(c.anchor).map_or(0.0,|anchor|t.transform_point(c.local_point).distance(anchor.translation));
                    format!("Câble {} · {:.0} m",if distance>c.length*0.98{"tendu"}else{"souple"},c.length)
                });
                format!("{:.0} / {:.0}  Aether\n{:.1} m/s   ·   {:.0} m d'altitude\n{:.0} kg   ·   {} cellules\nVoile {:.0}°   ·   vent {:.0} m/s\nMoteurs {:.0}% · courant {:.0}%\n{}\n\n{} · {:.0} m\nCap {} · {:.0}°\nQuai à {:.0} m d'altitude\n{}\n{}",v.fuel,v.body.fuel_capacity(),velocity.0.length(),t.translation.y,v.properties.mass,v.body.grid().len(),v.trim.to_degrees(),telemetry.wind.length(),telemetry.motor*100.0,telemetry.current_weight*100.0,if v.docked{"AMARRÉ / RECHARGE"}else if v.fuel<=0.001 || v.body.count_parts(PartKind::Lift) as f32*aether_core::tuning::LIFT_NEWTONS_PER_PART<v.properties.mass*aether_core::tuning::GRAVITY{"SUSTENTATION INSUFFISANTE"}else{"EN NAVIGATION"},aether_core::world::port_name(game.expedition.destination),to.length(),cap,angle.to_degrees().abs(),target.y,cable,if to.length()<35.0&&!v.docked{format!("Frein {} ; quai {}",prefs.bindings.label(crate::bindings::Control::Backward),prefs.bindings.label(crate::bindings::Control::Dock))}else{format!("{} : moteur · {} : atlas",prefs.bindings.label(crate::bindings::Control::Forward),prefs.bindings.label(crate::bindings::Control::Atlas))})
            })
        } else if objective.is_some() {
            interest.clone().or_else(|| tutorial
                .advice(prefs.tutorial, &prefs.bindings)
                .or_else(|| {
                    Some(
                match game.progress {
                    0 => "Ajoutez votre touche à la coque, puis larguez les amarres.".into(),
                    1 => "Entrez dans le courant lumineux. Gardez de l'Aether pour l'arrivée.".into(),
                    2 => format!("Les ancres dorées permettent de changer de trajectoire avec {}.",prefs.bindings.label(crate::bindings::Control::Tether)),
                    3 => format!("Libérez le câble, choisissez une escale dans l'atlas et amarrez-vous avec {}.",prefs.bindings.label(crate::bindings::Control::Dock)),
                    _ => "Les neuf archipels vous attendent. Ouvrez l'atlas pour choisir une escale et découvrir ses ressources.".into(),
                },
)
}))
        } else if selection.is_some() {
            Some(editor.selection())
        } else {
            None
        };
        if let Some(value) = value
            && text.0 != value
        {
            text.0 = value;
        }
    }
    if let Some((v, _, _, _)) = vessel {
        for mut bar in &mut fuel_bars {
            bar.width =
                percent((v.fuel / v.body.fuel_capacity().max(1.0) * 100.0).clamp(0.0, 100.0));
        }
    }
}
