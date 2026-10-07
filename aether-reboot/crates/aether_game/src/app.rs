use crate::{camera, controls, editor, interface, persistence, scenario, session};
use bevy::{asset::AssetPlugin, light::GlobalAmbientLight, prelude::*, window::PresentMode};

#[derive(States, Clone, Copy, Default, Debug, Eq, PartialEq, Hash)]
pub enum Phase {
    #[default]
    Boot,
    Loading,
    Error,
    Menu,
    Playing,
    Editing,
    Paused,
    Settings,
    Atlas,
    Travel,
}
#[derive(Resource, Default)]
pub struct LaunchOptions {
    pub smoke: bool,
    pub capture: Option<String>,
    pub elapsed: f32,
    pub captured: bool,
}
#[derive(Resource, Default)]
pub struct LoadingStatus {
    pub assets: Vec<(String, bevy::asset::UntypedHandle)>,
    pub elapsed: f32,
    pub error: String,
    /// Reload is queued asynchronously. Ignore only the exact old failures,
    /// until a new attempt replaces them or the normal loading timeout fires.
    pub retry_errors: Vec<std::sync::Arc<bevy::asset::AssetLoadError>>,
    pub retry_leaves: Vec<bevy::asset::UntypedAssetId>,
    pub retried_parents: Vec<bevy::asset::UntypedAssetId>,
    pub reset_timer: bool,
}
fn begin_loading(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut next: ResMut<NextState<Phase>>,
) {
    let mut handles = vec![
        (
            "shaders/clouds.wgsl".into(),
            assets
                .load::<bevy::shader::Shader>("shaders/clouds.wgsl")
                .untyped(),
        ),
        (
            "shaders/sky.wgsl".into(),
            assets
                .load::<bevy::shader::Shader>("shaders/sky.wgsl")
                .untyped(),
        ),
        (
            "textures/cedar-normal.png".into(),
            aether_view::materials::surface_texture(&assets, "textures/cedar-normal.png", false)
                .untyped(),
        ),
        (
            "textures/cedar.png".into(),
            aether_view::materials::wood_texture(&assets).untyped(),
        ),
        (
            "fonts/notosans.ttf".into(),
            assets.load::<Font>("fonts/notosans.ttf").untyped(),
        ),
        (
            "fonts/cormorantgaramond.ttf".into(),
            assets.load::<Font>("fonts/cormorantgaramond.ttf").untyped(),
        ),
    ];
    handles.push((
        aether_view::art::SKY_TEXTURE.into(),
        aether_view::art::panorama_texture(&assets).untyped(),
    ));
    handles.push((
        aether_view::world::FLOW_TEXTURE.into(),
        aether_view::world::flow_texture(&assets).untyped(),
    ));
    handles.push((
        aether_view::world::CURRENT_TEXTURE.into(),
        aether_view::world::current_texture(&assets).untyped(),
    ));
    handles.extend(aether_view::art::IBL_TEXTURES.map(|path| {
        (
            path.into(),
            aether_view::art::environment_texture(&assets, path).untyped(),
        )
    }));
    handles.extend(aether_view::clouds::banks().iter().map(|bank| {
        (
            format!("atmosphere/{}-shape.ktx2", bank.key),
            aether_view::clouds::shape_texture(&assets, bank).untyped(),
        )
    }));
    // The character is optional: avatar::spawn keeps its visible capsule when
    // Knight fails. Terrain/equipment/essential textures remain loading barriers.
    handles.extend(aether_view::art::SCENES.iter().map(|name| {
        let path = format!("art/{name}.glb");
        let handle = assets.load::<bevy::world_serialization::WorldAsset>(
            GltfAssetLabel::Scene(0).from_asset(path.clone()),
        );
        (path, handle.untyped())
    }));
    for name in aether_view::ship::MODELS {
        let path = format!("ship/{name}.glb");
        handles.push((
            path.clone(),
            assets
                .load::<bevy::world_serialization::WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset(path),
                )
                .untyped(),
        ));
    }
    for species in aether_core::fauna::Species::ALL {
        let path = format!("fauna/{}.glb", species.asset());
        let handle = assets.load::<bevy::world_serialization::WorldAsset>(
            GltfAssetLabel::Scene(0).from_asset(path.clone()),
        );
        handles.push((path, handle.untyped()));
    }
    for key in ["trader", "trader-lod"] {
        let path = format!("fauna/{key}.glb");
        let handle = assets.load::<bevy::world_serialization::WorldAsset>(
            GltfAssetLabel::Scene(0).from_asset(path.clone()),
        );
        handles.push((path, handle.untyped()));
    }
    for key in aether_core::world::ASSETS {
        for suffix in ["", "-lod"] {
            let path = format!("world/{key}{suffix}.glb");
            let handle = assets.load::<bevy::world_serialization::WorldAsset>(
                GltfAssetLabel::Scene(0).from_asset(path.clone()),
            );
            handles.push((path, handle.untyped()));
        }
    }
    for key in ["arch-vault", "arch-vault-lod"] {
        let path = format!("world/{key}.glb");
        let h = assets.load::<bevy::world_serialization::WorldAsset>(
            GltfAssetLabel::Scene(0).from_asset(path.clone()),
        );
        handles.push((path, h.untyped()));
    }
    handles.push((
        "shaders/world-flow.wgsl".into(),
        assets
            .load::<bevy::shader::Shader>("shaders/world-flow.wgsl")
            .untyped(),
    ));
    commands.insert_resource(LoadingStatus {
        assets: handles,
        ..default()
    });
    next.set(Phase::Loading);
}
fn observe_loading(
    assets: Res<AssetServer>,
    time: Res<Time<Real>>,
    mut status: ResMut<LoadingStatus>,
    mut next: ResMut<NextState<Phase>>,
) {
    if status.reset_timer {
        status.reset_timer = false;
        return;
    }
    status.elapsed += time.delta_secs();
    // Bevy retains failed dependency sets until the parent is loaded again.
    // Wait for failed leaves before rebuilding their glTF/scene dependents;
    // parallel reloads can otherwise retain the same failed dependency forever.
    if !status.retry_errors.is_empty()
        && status
            .retry_leaves
            .iter()
            .all(|id| assets.is_loaded_with_dependencies(*id))
    {
        let parents: Vec<_> = status
            .assets
            .iter()
            .filter_map(|(_, handle)| {
                let id = handle.id();
                if status.retried_parents.contains(&id) {
                    return None;
                }
                let Some(bevy::asset::RecursiveDependencyLoadState::Failed(error)) =
                    assets.get_recursive_dependency_load_state(id)
                else {
                    return None;
                };
                status
                    .retry_errors
                    .iter()
                    .any(|old| std::sync::Arc::ptr_eq(old, &error))
                    .then(|| handle.path().map(|path| (id, path.clone())))
                    .flatten()
            })
            .collect();
        for (id, path) in parents {
            assets.reload(path);
            status.retried_parents.push(id);
        }
    }
    let failed = status.assets.iter().find_map(|(path, handle)| {
        let error = match assets.get_load_state(handle.id()) {
            Some(bevy::asset::LoadState::Failed(error)) => Some(error),
            _ => match assets.get_recursive_dependency_load_state(handle.id()) {
                Some(bevy::asset::RecursiveDependencyLoadState::Failed(error)) => Some(error),
                _ => None,
            },
        }?;
        if status
            .retry_errors
            .iter()
            .any(|old| std::sync::Arc::ptr_eq(old, &error))
        {
            None
        } else {
            Some(format!("{path} : {error}"))
        }
    });
    if let Some(error) = failed {
        status.error = error;
        next.set(Phase::Error);
    } else if status
        .assets
        .iter()
        .all(|(_, h)| assets.is_loaded_with_dependencies(h.id()))
    {
        status.retry_errors.clear();
        status.retry_leaves.clear();
        status.retried_parents.clear();
        next.set(Phase::Menu);
    } else if status.elapsed > 30.0 {
        status.error =
            "Chargement trop long. Vérifiez que le dossier assets accompagne le jeu.".into();
        next.set(Phase::Error);
    }
}

pub fn run() -> AppExit {
    #[cfg(not(target_arch = "wasm32"))]
    let args: Vec<String> = std::env::args().collect();
    #[cfg(target_arch = "wasm32")]
    let args: Vec<String> = Vec::new();
    let capture = args
        .windows(2)
        .find(|a| a[0] == "--capture")
        .map(|a| a[1].clone());
    let smoke = args.iter().any(|a| a == "--smoke");
    #[cfg(target_arch = "wasm32")]
    let query = browser_options();
    #[cfg(not(target_arch = "wasm32"))]
    let query = String::new();
    let scripted = args.iter().any(|a| a == "--qa") || query.contains("qa=1");
    let benchmark = args.iter().any(|a| a == "--benchmark") || query.contains("bench=1");
    let probe = scripted || benchmark || query.contains("probe=1");
    let output = args
        .windows(2)
        .find(|a| a[0] == "--qa-output")
        .map_or_else(|| "docs/evidence/native-qa.json".into(), |a| a[1].clone());
    let mut diagnostics = crate::diagnostics::Diagnostics::new(scripted, probe, output);
    diagnostics.benchmark = benchmark;
    let mut app = App::new();
    app.insert_resource(diagnostics)
        .add_systems(Startup, crate::diagnostics::setup)
        .add_systems(FixedFirst, crate::diagnostics::tick_start)
        .add_systems(FixedLast, crate::diagnostics::tick_end);
    app.insert_resource(ClearColor(Color::srgb(0.12, 0.17, 0.25)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.68, 0.76, 1.0),
            brightness: 65.0,
            ..default()
        })
        .insert_resource(LaunchOptions {
            smoke,
            capture,
            ..default()
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: aether_view::asset_root(),
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    close_when_requested: cfg!(target_arch = "wasm32"),
                    primary_window: Some(Window {
                        title: "Aether Isles — Les neuf archipels".into(),
                        resolution: if benchmark {
                            bevy::window::WindowResolution::new(1920, 1080)
                                .with_scale_factor_override(1.0)
                        } else {
                            (1440, 900).into()
                        },
                        present_mode: PresentMode::AutoVsync,
                        canvas: Some("#aether-canvas".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_state::<Phase>()
        .add_systems(OnEnter(Phase::Boot), begin_loading)
        .add_systems(Update, observe_loading.run_if(in_state(Phase::Loading)))
        .add_plugins((
            aether_sim::SimulationPlugin,
            aether_sim::character::CharacterPlugin,
            aether_view::PresentationPlugin,
        ))
        .init_resource::<session::GameSession>()
        .init_resource::<controls::Actions>()
        .init_resource::<crate::bindings::BindingCapture>()
        .init_resource::<crate::travel::TravelMenu>()
        .init_resource::<interface::KeyboardUi>()
        .init_resource::<interface::PhotoMode>()
        .add_systems(Update, interface::photo.before(interface::keyboard))
        .init_resource::<crate::tutorial::Tutorial>()
        .add_systems(Update, crate::tutorial::update.after(controls::dispatch))
        .init_resource::<editor::Editor>()
        .init_resource::<camera::CameraRig>()
        .init_resource::<persistence::Storage>()
        .insert_resource(persistence::load_preferences())
        .add_systems(
            Startup,
            (camera::setup, session::setup)
                .chain()
                .after(aether_view::materials::setup)
                .after(aether_view::avatar::setup)
                .after(aether_view::art::setup)
                .after(aether_view::world::setup)
                .after(aether_view::fauna::setup)
                .after(aether_view::traffic::setup)
                .after(aether_view::weather::setup),
        )
        .add_systems(
            Update,
            (
                interface::keyboard,
                interface::buttons,
                controls::collect,
                controls::dispatch,
                crate::exploration::sync_payload,
                persistence::poll,
            )
                .chain(),
        )
        .add_systems(Update, (camera::control, camera::follow).chain())
        .add_systems(
            Update,
            (editor::pick, editor::apply)
                .chain()
                .run_if(in_state(Phase::Editing)),
        )
        .add_systems(
            Update,
            (
                scenario::update,
                scenario::draw_tether,
                interface::update_hud,
                aether_view::environment::animate_current,
                smoke_capture,
            ),
        )
        .add_systems(OnEnter(Phase::Menu), interface::rebuild)
        .add_systems(OnEnter(Phase::Loading), interface::rebuild)
        .add_systems(OnEnter(Phase::Error), interface::rebuild)
        .add_systems(OnEnter(Phase::Playing), interface::rebuild)
        .add_systems(OnEnter(Phase::Editing), interface::rebuild)
        .add_systems(OnEnter(Phase::Paused), interface::rebuild)
        .add_systems(OnEnter(Phase::Settings), interface::rebuild)
        .add_systems(OnEnter(Phase::Atlas), interface::rebuild)
        .add_systems(OnEnter(Phase::Travel), interface::rebuild)
        .add_systems(Update, (sync_pause, adapt_ui));
    app.configure_sets(
        Update,
        aether_view::PrepareVoxels
            .after(controls::dispatch)
            .after(persistence::poll)
            .after(editor::apply)
            .after(scenario::update),
    );
    if probe {
        app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);
    }
    app.add_systems(Startup, crate::audio::setup)
        .add_systems(
            PostUpdate,
            crate::diagnostics::verify_presentation.after(TransformSystems::Propagate),
        )
        .add_systems(Last, crate::diagnostics::update)
        .add_systems(Update, crate::scenario::animate_avatars)
        .add_systems(Update, crate::scenario::animate_equipment)
        .add_systems(Update, crate::scenario::animate_motors)
        .add_systems(
            Update,
            crate::scenario::weather.before(aether_view::weather::update),
        )
        .add_systems(Update, aether_sim::streaming::update)
        .add_systems(Update, aether_sim::streaming::vault_update)
        .add_systems(
            Update,
            crate::exploration::observe.run_if(in_state(Phase::Playing)),
        )
        .add_systems(Update, crate::audio::signals)
        .add_systems(OnEnter(Phase::Menu), web_ready)
        .add_systems(OnEnter(Phase::Error), web_ready);
    #[cfg(not(target_arch = "wasm32"))]
    app.init_resource::<crate::closing::Closing>().add_systems(
        Update,
        (crate::closing::collect, crate::closing::finish)
            .chain()
            .after(persistence::poll),
    );
    app.run()
}
fn web_ready() {
    #[cfg(target_arch = "wasm32")]
    browser_ready();
}
fn adapt_ui(windows: Query<&Window>, mut scale: ResMut<UiScale>) {
    if let Ok(window) = windows.single() {
        let value = (window.width() / 1440.0)
            .min(window.height() / 900.0)
            .clamp(0.5, 1.3);
        if (scale.0 - value).abs() > 0.001 {
            scale.0 = value;
        }
    }
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function browser_ready(){ window.aetherReady?.(); } export function browser_options(){ return location.search; }"
)]
extern "C" {
    fn browser_ready();
    fn browser_options() -> String;
}
fn sync_pause(
    phase: Res<State<Phase>>,
    windows: Query<&Window>,
    mut time: ResMut<Time<Virtual>>,
    diagnostics: Res<crate::diagnostics::Diagnostics>,
) {
    let focused = diagnostics.benchmark || windows.single().is_ok_and(|w| w.focused);
    if !focused || !matches!(phase.get(), Phase::Playing | Phase::Editing) {
        time.pause();
    } else {
        time.unpause();
    }
    time.set_max_delta(std::time::Duration::from_secs_f64(0.1));
}
fn smoke_capture(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut options: ResMut<LaunchOptions>,
    metrics: Res<aether_view::MeshMetrics>,
    phase: Res<State<Phase>>,
    mut exit: MessageWriter<AppExit>,
) {
    options.elapsed += time.delta_secs();
    if !options.captured && options.elapsed > 8.0 && metrics.pending == 0 {
        if let Some(path) = &options.capture {
            commands
                .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(path.clone()));
        }
        options.captured = true;
    }
    if options.smoke && options.elapsed > 12.0 {
        exit.write(if *phase.get() == Phase::Menu {
            AppExit::Success
        } else {
            AppExit::error()
        });
    }
}
