#![cfg(not(target_arch = "wasm32"))]
use aether_game::{persistence::native, session};
#[test]
fn atomic_save_and_backup_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let first = session::starter();
    native::save_in(dir.path(), &first.encode().unwrap()).unwrap();
    let mut next = first.clone();
    next.progress = 2;
    native::save_in(dir.path(), &next.encode().unwrap()).unwrap();
    assert_eq!(native::load_in(dir.path()).unwrap().0.progress, 2);
    std::fs::write(dir.path().join("session.json"), b"truncated").unwrap();
    let (restored, backup) = native::load_in(dir.path()).unwrap();
    assert!(backup);
    assert_eq!(restored.progress, 0);
}
#[test]
fn invalid_save_cannot_replace_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    native::save_in(dir.path(), &session::starter().encode().unwrap()).unwrap();
    assert!(native::save_in(dir.path(), b"{}").is_err());
    assert!(native::load_in(dir.path()).is_ok());
}
#[test]
fn corrupt_primary_does_not_overwrite_valid_backup() {
    let dir = tempfile::tempdir().unwrap();
    let first = session::starter();
    native::save_in(dir.path(), &first.encode().unwrap()).unwrap();
    native::save_in(dir.path(), &first.encode().unwrap()).unwrap();
    std::fs::write(dir.path().join("session.json"), b"corrupt").unwrap();
    let mut next = first;
    next.progress = 3;
    native::save_in(dir.path(), &next.encode().unwrap()).unwrap();
    std::fs::write(dir.path().join("session.json"), b"corrupt again").unwrap();
    assert_eq!(native::load_in(dir.path()).unwrap().0.progress, 0);
}
#[test]
fn isolated_directory_needs_no_checkout() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("portable-user-data");
    native::save_in(&nested, &session::starter().encode().unwrap()).unwrap();
    assert!(native::load_in(&nested).is_ok());
}

#[test]
fn interrupted_save_at_each_durable_write_boundary_keeps_old_or_new_session() {
    for stop in 0..12 {
        let dir = tempfile::tempdir().unwrap();
        let mut old = session::starter();
        old.progress = 1;
        native::save_in(dir.path(), &old.encode().unwrap()).unwrap();
        let mut next = old.clone();
        next.progress = 2;
        let mut observed = 0;
        let result = native::save_in_observed(dir.path(), &next.encode().unwrap(), |_, stage| {
            let index = observed;
            observed += 1;
            if index == stop {
                Err(format!("interruption injectée : {stage:?}"))
            } else {
                Ok(())
            }
        });
        assert!(result.is_err(), "boundary {stop} must have been reached");
        let (restored, _) = native::load_in(dir.path()).unwrap();
        assert!([1, 2].contains(&restored.progress), "boundary {stop}");
        assert_eq!(
            restored.vessels[0].blueprint.cells.len(),
            old.vessels[0].blueprint.cells.len()
        );
        if let Ok(bytes) = std::fs::read(dir.path().join("session.backup.json")) {
            assert_eq!(
                aether_core::save::Session::decode(&bytes).unwrap().progress,
                1
            );
        }
    }
}

#[test]
fn closing_after_snapshot_failure_does_not_reuse_an_old_success() {
    use aether_game::{app::Phase, closing, persistence::Storage};
    use bevy::{prelude::*, state::app::StatesPlugin, window::WindowCloseRequested};
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Phase>()
        .init_resource::<Storage>()
        .init_resource::<session::GameSession>()
        .init_resource::<closing::Closing>()
        .add_message::<WindowCloseRequested>()
        .add_systems(Update, (closing::collect, closing::finish).chain());
    app.world_mut()
        .resource_mut::<NextState<Phase>>()
        .set(Phase::Playing);
    app.update();
    app.world_mut().resource_mut::<Storage>().last_save_ok = true;
    // No active vessel: snapshot fails before a disk operation can start.
    app.world_mut().write_message(WindowCloseRequested {
        window: Entity::PLACEHOLDER,
    });
    app.update();
    app.update();
    assert!(!app.world().resource::<Storage>().last_save_ok);
    assert!(
        app.world()
            .resource::<session::GameSession>()
            .notice
            .contains("Fermez de nouveau")
    );
    assert!(app.world().resource::<Messages<AppExit>>().is_empty());
    app.world_mut().write_message(WindowCloseRequested {
        window: Entity::PLACEHOLDER,
    });
    app.update();
    assert!(!app.world().resource::<Messages<AppExit>>().is_empty());
}
