use aether_game::{
    app::Phase,
    bindings::{BindingCapture, BindingKey, Control},
    camera::CameraRig,
    controls::{self, Action, Actions},
    interface::KeyboardUi,
    persistence::Preferences,
    session::GameSession,
};
use bevy::{ecs::system::RunSystemOnce, input::keyboard::Key, prelude::*};
use std::collections::VecDeque;

fn capture_frame(consumed: bool) -> World {
    let mut world = World::new();
    world.insert_resource(State::new(Phase::Settings));
    world.insert_resource(GameSession::default());
    world.insert_resource(CameraRig::default());
    world.insert_resource(Preferences::default());
    world.insert_resource(BindingCapture::default());
    world.insert_resource(KeyboardUi {
        consumed,
        ..default()
    });
    world.insert_resource(Actions(VecDeque::from([Action::Rebind(Control::Ascend)])));
    world.insert_resource(ButtonInput::<KeyCode>::default());
    world.insert_resource(ButtonInput::<Key>::default());
    world.spawn(Window {
        focused: true,
        ..default()
    });
    world
}

#[test]
fn click_and_key_in_one_frame_queue_capture_before_assignment() {
    // The real button system has queued Rebind; its dispatcher runs after
    // collection. A key in this same frame must not require a second press.
    let mut world = capture_frame(false);
    world.resource_mut::<Actions>().0.clear();
    let button = world
        .spawn((
            Interaction::Pressed,
            aether_game::interface::UiAction(Action::Rebind(Control::Ascend)),
            aether_game::interface::ButtonStyle(false),
            BackgroundColor(Color::NONE),
        ))
        .id();
    world.spawn((TextColor(Color::WHITE), ChildOf(button)));
    world
        .run_system_once(aether_game::interface::buttons)
        .unwrap();
    world
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    world.resource_mut::<ButtonInput<Key>>().press(Key::Shift);
    world.run_system_once(controls::collect).unwrap();
    assert_eq!(
        world.resource::<Actions>().0,
        VecDeque::from([
            Action::Rebind(Control::Ascend),
            Action::SetBinding(BindingKey::Shift),
        ])
    );
}

#[test]
fn enter_used_to_activate_the_button_is_not_also_captured() {
    let mut world = capture_frame(true);
    world
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);
    world.resource_mut::<ButtonInput<Key>>().press(Key::Enter);
    world.run_system_once(controls::collect).unwrap();
    assert_eq!(
        world.resource::<Actions>().0,
        VecDeque::from([Action::Rebind(Control::Ascend)])
    );
}

#[test]
fn updating_assignment_keeps_the_row_root_and_keyboard_focus() {
    let mut world = capture_frame(false);
    let root = world.spawn(aether_game::interface::UiRoot).id();
    let row = world
        .spawn((
            aether_game::interface::UiAction(Action::Rebind(Control::Ascend)),
            aether_game::interface::ButtonStyle(false),
            BackgroundColor(Color::NONE),
            ChildOf(root),
        ))
        .id();
    let child = world
        .spawn((Text::new("Monter"), TextColor(Color::WHITE), ChildOf(row)))
        .id();
    world.resource_mut::<KeyboardUi>().focused = Some(row);
    world.resource_mut::<KeyboardUi>().active = true;
    world.resource_mut::<BindingCapture>().selected = Some(Control::Ascend);
    world.resource_mut::<BindingCapture>().active = Some(Control::Ascend);
    let entities = world.entity_count();
    aether_game::interface::sync_bindings(&mut world);
    world
        .resource_mut::<Preferences>()
        .bindings
        .assign(Control::Ascend, BindingKey::Shift)
        .unwrap();
    world.resource_mut::<BindingCapture>().active = None;
    aether_game::interface::sync_bindings(&mut world);
    assert_eq!(world.entity_count(), entities);
    assert!(world.get_entity(root).is_ok());
    assert_eq!(world.resource::<KeyboardUi>().focused, Some(row));
    assert!(
        world
            .get::<aether_game::interface::ButtonStyle>(row)
            .unwrap()
            .0
    );
    assert_eq!(world.get::<Text>(child).unwrap().0, "Monter : Maj");
}
