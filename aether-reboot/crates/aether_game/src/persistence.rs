use crate::{
    app::Phase,
    controls::notice,
    session::{self, GameSession},
};
use aether_core::{
    Body,
    save::{BlueprintFile, Session},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub volume: f32,
    pub sensitivity: f32,
    pub invert_y: bool,
    pub reduce_motion: bool,
    pub bindings: crate::bindings::Bindings,
    pub tutorial: u16,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            volume: 0.6,
            sensitivity: 1.0,
            invert_y: false,
            reduce_motion: false,
            bindings: default(),
            tutorial: 0,
        }
    }
}
impl Preferences {
    fn validate(&mut self) {
        if !self.volume.is_finite() {
            self.volume = 0.6;
        }
        if !self.sensitivity.is_finite() {
            self.sensitivity = 1.0;
        }
        self.volume = self.volume.clamp(0.0, 1.0);
        self.sensitivity = self.sensitivity.clamp(0.25, 2.0);
        self.bindings.validate();
        self.tutorial &= crate::tutorial::COMPLETE;
    }
}
enum Outcome {
    Saved,
    Loaded(Session, bool),
    Exported,
    Imported(Body),
    Cancelled,
}
type Mailbox = Arc<Mutex<VecDeque<Result<Outcome, String>>>>;
#[derive(Resource, Default)]
pub struct Storage {
    pub pending: bool,
    pub last_save_ok: bool,
    mailbox: Mailbox,
}
fn begin(world: &mut World) -> Option<Mailbox> {
    let mut storage = world.resource_mut::<Storage>();
    if storage.pending {
        return None;
    }
    storage.pending = true;
    storage.last_save_ok = false;
    Some(storage.mailbox.clone())
}
fn deliver(mailbox: &Mailbox, result: Result<Outcome, String>) {
    if let Ok(mut messages) = mailbox.lock() {
        messages.push_back(result);
    }
}

pub fn save(world: &mut World) {
    if world.resource::<Storage>().pending {
        return;
    }
    world.resource_mut::<Storage>().last_save_ok = false;
    let bytes = match session::snapshot(world).and_then(|s| s.encode().map_err(|e| e.to_string())) {
        Ok(bytes) => bytes,
        Err(error) => {
            notice(world, error);
            return;
        }
    };
    let Some(mailbox) = begin(world) else {
        return;
    };
    notice(world, "Sauvegarde en cours…");
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        deliver(&mailbox, native::save(&bytes).map(|()| Outcome::Saved));
    });
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let payload = String::from_utf8(bytes).expect("JSON utf8");
        let valid_old = web::call("load", "")
            .await
            .is_ok_and(|json| Session::decode(json.as_bytes()).is_ok());
        deliver(
            &mailbox,
            web::call(if valid_old { "save" } else { "save_no_backup" }, &payload)
                .await
                .map(|_| Outcome::Saved),
        );
    });
}
pub fn load(world: &mut World) {
    let Some(mailbox) = begin(world) else {
        return;
    };
    notice(world, "Lecture de la sauvegarde…");
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        deliver(&mailbox, native::load().map(|(s, b)| Outcome::Loaded(s, b)));
    });
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let result = match web::call("load", "").await {
            Ok(json) => Session::decode(json.as_bytes())
                .map(|s| Outcome::Loaded(s, false))
                .map_err(|e| e.to_string()),
            Err(e) => Err(e),
        };
        let result = match result {
            Ok(r) => Ok(r),
            Err(first) => match web::call("backup", "").await {
                Ok(json) => Session::decode(json.as_bytes())
                    .map(|s| Outcome::Loaded(s, true))
                    .map_err(|_| first),
                Err(_) => Err(first),
            },
        };
        deliver(&mailbox, result);
    });
}
pub fn export(world: &mut World) {
    let Some(entity) = world.resource::<GameSession>().active else {
        return;
    };
    let Some(vessel) = world.get::<aether_sim::Vessel>(entity) else {
        return;
    };
    let bytes = match BlueprintFile::encode(&vessel.body) {
        Ok(v) => v,
        Err(e) => {
            notice(world, e.to_string());
            return;
        }
    };
    let Some(mailbox) = begin(world) else {
        return;
    };
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        let result = if let Some(path) = rfd::FileDialog::new()
            .add_filter("Construction Aether", &["json"])
            .set_file_name("alcyon.aether.json")
            .save_file()
        {
            native::atomic_write(&path, &bytes).map(|_| Outcome::Exported)
        } else {
            Ok(Outcome::Cancelled)
        };
        deliver(&mailbox, result);
    });
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        deliver(
            &mailbox,
            web::call("export", &String::from_utf8(bytes).expect("JSON"))
                .await
                .map(|_| Outcome::Exported),
        );
    });
}
pub fn import(world: &mut World) {
    let Some(entity) = world.resource::<GameSession>().active else {
        return;
    };
    if !world
        .get::<aether_sim::Vessel>(entity)
        .is_some_and(|v| v.docked)
    {
        notice(world, "L'import d'une construction s'effectue au quai.");
        return;
    }
    let Some(mailbox) = begin(world) else {
        return;
    };
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        let result = if let Some(path) = rfd::FileDialog::new()
            .add_filter("Construction Aether", &["json"])
            .pick_file()
        {
            native::read_limited(&path).and_then(|data| {
                BlueprintFile::decode(&data)
                    .map(Outcome::Imported)
                    .map_err(|e| e.to_string())
            })
        } else {
            Ok(Outcome::Cancelled)
        };
        deliver(&mailbox, result);
    });
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let result = web::call("import", "").await.and_then(|data| {
            if data.is_empty() {
                Ok(Outcome::Cancelled)
            } else {
                BlueprintFile::decode(data.as_bytes())
                    .map(Outcome::Imported)
                    .map_err(|e| e.to_string())
            }
        });
        deliver(&mailbox, result);
    });
}
pub fn poll(world: &mut World) {
    let message = world
        .resource::<Storage>()
        .mailbox
        .lock()
        .ok()
        .and_then(|mut q| q.pop_front());
    let Some(message) = message else {
        return;
    };
    world.resource_mut::<Storage>().pending = false;
    match message {
        Ok(Outcome::Saved) => {
            world.resource_mut::<Storage>().last_save_ok = true;
            notice(world, "Sauvegarde enregistrée.");
        }
        Ok(Outcome::Loaded(session, recovered)) => {
            session::replace(world, session);
            world.resource_mut::<NextState<Phase>>().set(Phase::Playing);
            notice(
                world,
                if recovered {
                    "Copie de secours restaurée."
                } else {
                    "Traversée reprise."
                },
            );
        }
        Ok(Outcome::Imported(body)) => {
            let Some(entity) = world.resource::<GameSession>().active else {
                return;
            };
            if let Some(mut vessel) = world.get_mut::<aether_sim::Vessel>(entity) {
                // Keep revisions monotonic so an old mesh job cannot publish after an import.
                vessel
                    .body
                    .replace(body.blueprint())
                    .expect("validated import");
                vessel.fuel = vessel.fuel.min(vessel.body.fuel_capacity());
            }
            world.resource_mut::<crate::editor::Editor>().reset();
            crate::editor::refresh(world, entity, vec![]);
            notice(world, "Construction importée. Réserve d'Aether conservée.");
        }
        Ok(Outcome::Exported) => notice(world, "Construction exportée."),
        Ok(Outcome::Cancelled) => notice(world, "Opération annulée."),
        Err(error) => notice(world, format!("Opération impossible : {error}")),
    }
}
pub fn load_preferences() -> Preferences {
    #[cfg(not(target_arch = "wasm32"))]
    let json = std::fs::read_to_string(native::directory().join("preferences.json")).ok();
    #[cfg(target_arch = "wasm32")]
    let json = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("aether-preferences-v1").ok().flatten());
    let mut prefs = json
        .and_then(|s| serde_json::from_str::<Preferences>(&s).ok())
        .unwrap_or_default();
    prefs.validate();
    prefs
}
pub fn save_preferences(world: &mut World) {
    let prefs = world.resource::<Preferences>();
    let Ok(json) = serde_json::to_string_pretty(prefs) else {
        return;
    };
    #[cfg(not(target_arch = "wasm32"))]
    let result = native::atomic_write(
        &native::directory().join("preferences.json"),
        json.as_bytes(),
    );
    #[cfg(target_arch = "wasm32")]
    let result = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .ok_or_else(|| "Stockage indisponible".to_string())
        .and_then(|s| {
            s.set_item("aether-preferences-v1", &json)
                .map_err(|e| format!("{e:?}"))
        });
    if let Err(error) = result {
        notice(world, format!("Préférences non enregistrées : {error}"));
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use aether_core::{MAX_SAVE_BYTES, save::Session};
    use std::{
        fs,
        io::{Read, Write},
        path::{Path, PathBuf},
    };
    pub fn directory() -> PathBuf {
        std::env::var_os("AETHER_DATA")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::args()
                    .any(|a| a == "--qa" || a == "--benchmark")
                    .then(|| {
                        std::env::temp_dir()
                            .join("aether-reboot-checks")
                            .join(std::process::id().to_string())
                    })
            })
            .or_else(|| {
                directories::ProjectDirs::from("games", "AetherIsles", "Reboot")
                    .map(|p| p.data_local_dir().to_path_buf())
            })
            .unwrap_or_else(|| PathBuf::from("saves"))
    }
    pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
        atomic_write_observed(path, bytes, &mut |_, _| Ok(()))
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WriteStage {
        Directory,
        Temporary,
        PartialWrite,
        CompleteWrite,
        Synced,
        Replaced,
    }
    fn atomic_write_observed(
        path: &Path,
        bytes: &[u8],
        observer: &mut impl FnMut(&Path, WriteStage) -> Result<(), String>,
    ) -> Result<(), String> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        observer(path, WriteStage::Directory)?;
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        observer(path, WriteStage::Temporary)?;
        let (first, second) = bytes.split_at(bytes.len() / 2);
        file.write_all(first).map_err(|e| e.to_string())?;
        observer(path, WriteStage::PartialWrite)?;
        file.write_all(second).map_err(|e| e.to_string())?;
        observer(path, WriteStage::CompleteWrite)?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        observer(path, WriteStage::Synced)?;
        file.persist(path).map_err(|e| e.to_string())?;
        observer(path, WriteStage::Replaced)?;
        Ok(())
    }
    pub fn read_limited(path: &Path) -> Result<Vec<u8>, String> {
        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        if file.metadata().map_err(|e| e.to_string())?.len() > MAX_SAVE_BYTES as u64 {
            return Err("Fichier trop volumineux".into());
        }
        let mut bytes = Vec::new();
        file.take(MAX_SAVE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > MAX_SAVE_BYTES {
            return Err("Fichier trop volumineux".into());
        }
        Ok(bytes)
    }
    pub fn save(bytes: &[u8]) -> Result<(), String> {
        save_in(&directory(), bytes)
    }
    pub fn save_in(directory: &Path, bytes: &[u8]) -> Result<(), String> {
        save_in_observed(directory, bytes, |_, _| Ok(()))
    }
    /// Same production transaction, with an observer for deterministic fault injection.
    pub fn save_in_observed(
        directory: &Path,
        bytes: &[u8],
        mut observer: impl FnMut(&Path, WriteStage) -> Result<(), String>,
    ) -> Result<(), String> {
        Session::decode(bytes).map_err(|e| e.to_string())?;
        let main = directory.join("session.json");
        if let Ok(old) = read_limited(&main)
            && Session::decode(&old).is_ok()
        {
            atomic_write_observed(&directory.join("session.backup.json"), &old, &mut observer)?;
        }
        atomic_write_observed(&main, bytes, &mut observer)
    }
    pub fn load() -> Result<(Session, bool), String> {
        load_in(&directory())
    }
    pub fn load_in(directory: &Path) -> Result<(Session, bool), String> {
        let first = read_limited(&directory.join("session.json"))
            .and_then(|b| Session::decode(&b).map_err(|e| e.to_string()));
        match first {
            Ok(s) => Ok((s, false)),
            Err(first) => read_limited(&directory.join("session.backup.json"))
                .and_then(|b| {
                    Session::decode(&b)
                        .map(|s| (s, true))
                        .map_err(|e| e.to_string())
                })
                .map_err(|_| format!("Aucune sauvegarde valide : {first}")),
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::prelude::*;
    // Browser platform adapter only. Simulation, validation, UI and codecs remain Rust.
    #[wasm_bindgen(inline_js = r#"
async function db() {
  return await new Promise((resolve,reject)=>{
    const request=indexedDB.open('aether-isles-reboot',1);
    request.onupgradeneeded=()=>request.result.createObjectStore('saves');
    request.onsuccess=()=>resolve(request.result);
    request.onerror=()=>reject(request.error);
    request.onblocked=()=>reject(new Error('Fermez les autres onglets Aether.'));
  });
}
export async function aether_storage(operation,payload) {
  if(operation==='export') {
    const url=URL.createObjectURL(new Blob([payload],{type:'application/json'}));
    const a=document.createElement('a'); a.href=url;a.download='alcyon.aether.json';
    document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);return '';
  }
  if(operation==='import') {
    return await new Promise((resolve,reject)=>{
      const input=document.createElement('input');input.type='file';input.accept='.json';input.hidden=true;document.body.append(input);
      input.oncancel=()=>{input.remove();resolve('');};
      input.onchange=async()=>{try{const f=input.files[0];if(!f){resolve('');return;}if(f.size>16777216)throw new Error('Fichier trop volumineux');resolve(await f.text());}catch(e){reject(e);}finally{input.remove();}};
      input.click();
    });
  }
  const database=await db();
  try {
    return await new Promise((resolve,reject)=>{
      const tx=database.transaction('saves',operation.startsWith('save')?'readwrite':'readonly',{durability:'strict'});const store=tx.objectStore('saves');let result='';
      tx.oncomplete=()=>resolve(result);tx.onabort=()=>reject(tx.error||new Error('Transaction annulée'));tx.onerror=()=>reject(tx.error);
      if(operation.startsWith('save')) {const old=store.get('session');old.onsuccess=()=>{if(operation==='save'&&old.result)store.put(old.result,'backup');store.put(payload,'session');};}
      else {const get=store.get(operation==='backup'?'backup':'session');get.onsuccess=()=>{result=get.result||'';};}
    });
  } finally {database.close();}
}
"#)]
    extern "C" {
        #[wasm_bindgen(catch)]
        async fn aether_storage(operation: &str, payload: &str) -> Result<JsValue, JsValue>;
    }
    pub async fn call(op: &str, payload: &str) -> Result<String, String> {
        aether_storage(op, payload)
            .await
            .map(|s| s.as_string().unwrap_or_default())
            .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))
    }
}
