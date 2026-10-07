//! Reproducible examples of actual production codecs.
fn main() {
    let root = std::path::Path::new("docs/examples-data");
    std::fs::create_dir_all(root).unwrap();
    let mut session = aether_game::session::starter();
    session.vessels[0].position = bevy::prelude::Vec3::new(13.0, 28.0, -99.0);
    session.vessels[0].rotation = bevy::prelude::Quat::from_rotation_y(0.87);
    session.vessels[0].target_altitude = Some(31.0);
    session.vessels[0].docked = false;
    session.progress = 2;
    std::fs::write(root.join("session-v1.json"), session.encode().unwrap()).unwrap();
    let body = aether_core::fixtures::starter();
    std::fs::write(
        root.join("blueprint-v1.json"),
        aether_core::save::BlueprintFile::encode(&body).unwrap(),
    )
    .unwrap();
    let mut early = serde_json::to_value(&session).unwrap();
    early["vessels"][0]
        .as_object_mut()
        .unwrap()
        .remove("target_altitude");
    early.as_object_mut().unwrap().remove("walker");
    std::fs::write(
        root.join("session-early-v1.json"),
        serde_json::to_vec_pretty(&early).unwrap(),
    )
    .unwrap();
}
