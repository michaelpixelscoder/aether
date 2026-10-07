//! User bindings use printed characters, so a saved key keeps its meaning on AZERTY.
use bevy::{input::keyboard::Key, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Control {
    Forward,
    Backward,
    Left,
    Right,
    Ascend,
    Descend,
    TrimLeft,
    TrimRight,
    Jump,
    Dock,
    Tether,
    Walk,
    Edit,
    Recover,
    Rotate,
    Atlas,
    Interact,
    Reverse,
}
impl Control {
    pub const ALL: [Self; 18] = [
        Self::Forward,
        Self::Backward,
        Self::Left,
        Self::Right,
        Self::Ascend,
        Self::Descend,
        Self::TrimLeft,
        Self::TrimRight,
        Self::Jump,
        Self::Dock,
        Self::Tether,
        Self::Walk,
        Self::Edit,
        Self::Recover,
        Self::Rotate,
        Self::Atlas,
        Self::Interact,
        Self::Reverse,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Forward => "Avancer / moteur",
            Self::Backward => "Freiner / reculer",
            Self::Left => "Gauche",
            Self::Right => "Droite",
            Self::Ascend => "Monter",
            Self::Descend => "Descendre",
            Self::TrimLeft => "Voile −",
            Self::TrimRight => "Voile +",
            Self::Jump => "Sauter",
            Self::Dock => "Amarrer / partir",
            Self::Tether => "Harpon",
            Self::Walk => "Marcher / piloter",
            Self::Edit => "Atelier",
            Self::Recover => "Secours",
            Self::Rotate => "Tourner la pièce",
            Self::Atlas => "Atlas et soute",
            Self::Interact => "Interagir / récolter",
            Self::Reverse => "Moteur en marche arrière",
        }
    }
    fn defaults(self) -> &'static [BindingKey] {
        use BindingKey::*;
        match self {
            Self::Forward => &[Letter('z'), Letter('w'), Up],
            Self::Backward => &[Letter('s'), Down],
            Self::Left => &[Letter('q'), Letter('a'), Left],
            Self::Right => &[Letter('d'), Right],
            Self::Ascend => &[Letter('e')],
            Self::Descend => &[Letter('c')],
            Self::TrimLeft => &[Letter('j')],
            Self::TrimRight => &[Letter('x')],
            Self::Jump => &[Space],
            Self::Dock => &[Letter('f')],
            Self::Tether => &[Letter('g')],
            Self::Walk => &[Letter('t')],
            Self::Edit => &[Tab],
            Self::Recover => &[Letter('r')],
            Self::Rotate => &[Letter('v')],
            Self::Atlas => &[Letter('m')],
            Self::Interact => &[Letter('b')],
            Self::Reverse => &[Letter('k')],
        }
    }
    // Walking-only and sailing-only controls can legitimately share a key.
    fn overlaps(self, other: Self) -> bool {
        !matches!(
            (self, other),
            (
                Self::Jump,
                Self::Ascend | Self::Descend | Self::TrimLeft | Self::TrimRight
            ) | (
                Self::Ascend | Self::Descend | Self::TrimLeft | Self::TrimRight,
                Self::Jump
            )
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingKey {
    Letter(char),
    Space,
    Tab,
    Shift,
    Left,
    Right,
    Up,
    Down,
}
impl BindingKey {
    pub fn label(self) -> String {
        match self {
            Self::Letter(c) => c.to_uppercase().collect(),
            Self::Space => "Espace".into(),
            Self::Tab => "Tab".into(),
            Self::Shift => "Maj".into(),
            Self::Left => "Flèche gauche".into(),
            Self::Right => "Flèche droite".into(),
            Self::Up => "Flèche haut".into(),
            Self::Down => "Flèche bas".into(),
        }
    }
    pub fn from_logical(key: &Key) -> Option<Self> {
        Some(match key {
            Key::Character(s) if s.len() == 1 && s.as_bytes()[0].is_ascii_alphabetic() => {
                Self::Letter(s.chars().next()?.to_ascii_lowercase())
            }
            Key::Space => Self::Space,
            Key::Tab => Self::Tab,
            Key::Shift => Self::Shift,
            Key::ArrowLeft => Self::Left,
            Key::ArrowRight => Self::Right,
            Key::ArrowUp => Self::Up,
            Key::ArrowDown => Self::Down,
            _ => return None,
        })
    }
    fn pressed(
        self,
        physical: &ButtonInput<KeyCode>,
        logical: &ButtonInput<Key>,
        just: bool,
    ) -> bool {
        let code = match self {
            Self::Letter(c) => {
                return if just {
                    logical
                        .get_just_pressed()
                        .any(|k| Self::from_logical(k) == Some(Self::Letter(c)))
                } else {
                    logical
                        .get_pressed()
                        .any(|k| Self::from_logical(k) == Some(Self::Letter(c)))
                };
            }
            Self::Space => KeyCode::Space,
            Self::Tab => KeyCode::Tab,
            Self::Shift => {
                return [KeyCode::ShiftLeft, KeyCode::ShiftRight]
                    .into_iter()
                    .any(|key| {
                        if just {
                            physical.just_pressed(key)
                        } else {
                            physical.pressed(key)
                        }
                    });
            }
            Self::Left => KeyCode::ArrowLeft,
            Self::Right => KeyCode::ArrowRight,
            Self::Up => KeyCode::ArrowUp,
            Self::Down => KeyCode::ArrowDown,
        };
        if just {
            physical.just_pressed(code)
        } else {
            physical.pressed(code)
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Bindings(pub BTreeMap<Control, BindingKey>);
impl Bindings {
    pub fn keys(&self, control: Control) -> &[BindingKey] {
        self.0
            .get(&control)
            .map_or_else(|| control.defaults(), std::slice::from_ref)
    }
    pub fn label(&self, control: Control) -> String {
        self.keys(control)
            .iter()
            .map(|k| k.label())
            .collect::<Vec<_>>()
            .join(" / ")
    }
    pub fn pressed(
        &self,
        control: Control,
        physical: &ButtonInput<KeyCode>,
        logical: &ButtonInput<Key>,
        just: bool,
    ) -> bool {
        self.keys(control)
            .iter()
            .any(|k| k.pressed(physical, logical, just))
    }
    pub fn assign(&mut self, control: Control, key: BindingKey) -> Result<(), String> {
        if matches!(key,BindingKey::Letter(c) if !c.is_ascii_lowercase()) {
            return Err("Utilisez une lettre A–Z, une flèche, Espace ou Tab.".into());
        }
        if let Some(conflict) = self.conflict(control, key) {
            return Err(format!(
                "{} est déjà affectée à « {} ».",
                key.label(),
                conflict.label()
            ));
        }
        self.0.insert(control, key);
        Ok(())
    }
    pub fn conflict(&self, control: Control, key: BindingKey) -> Option<Control> {
        Control::ALL
            .into_iter()
            .find(|c| *c != control && c.overlaps(control) && self.keys(*c).contains(&key))
    }
    /// Exchange primary keys only after an explicit user action. Validate both
    /// assignments on a copy: context-sharing can introduce a third conflict.
    pub fn swap(&mut self, control: Control, key: BindingKey) -> Result<(), String> {
        let Some(other) = self.conflict(control, key) else {
            return self.assign(control, key);
        };
        let old_key = self.keys(control)[0];
        let mut candidate = self.clone();
        candidate.0.insert(control, key);
        candidate.assign(other, old_key)?;
        candidate.assign(control, key)?;
        *self = candidate;
        Ok(())
    }
    pub fn validate(&mut self) {
        if self
            .0
            .iter()
            .any(|(_, k)| matches!(k,BindingKey::Letter(c) if !c.is_ascii_lowercase()))
        {
            self.0.clear();
            return;
        }
        if Control::ALL.iter().any(|a| {
            Control::ALL.iter().any(|b| {
                a < b && a.overlaps(*b) && self.keys(*a).iter().any(|k| self.keys(*b).contains(k))
            })
        }) {
            self.0.clear();
        }
    }
}
#[derive(Resource, Default)]
pub struct BindingCapture {
    /// Keep the edited row highlighted after capture completes.
    pub selected: Option<Control>,
    pub active: Option<Control>,
    pub conflict: Option<(BindingKey, Control)>,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remap_is_persistent_logical_and_conflicts_are_rejected() {
        let mut bindings = Bindings::default();
        assert!(
            bindings
                .assign(Control::Dock, BindingKey::Letter('g'))
                .is_err()
        );
        bindings
            .assign(Control::Dock, BindingKey::Letter('h'))
            .unwrap();
        let mut restored: Bindings =
            serde_json::from_str(&serde_json::to_string(&bindings).unwrap()).unwrap();
        restored.validate();
        let mut keys = ButtonInput::default();
        keys.press(Key::Character("H".into()));
        assert!(restored.pressed(Control::Dock, &ButtonInput::default(), &keys, true));
        keys.reset_all();
        keys.press(Key::Character("f".into()));
        assert!(!restored.pressed(Control::Dock, &ButtonInput::default(), &keys, true));
        let old: crate::persistence::Preferences = serde_json::from_str(
            r#"{"volume":0.6,"sensitivity":1.0,"invert_y":false,"reduce_motion":false}"#,
        )
        .unwrap();
        assert_eq!(old.bindings.label(Control::Dock), "F");
    }
    #[test]
    fn shift_captures_and_works_with_either_physical_key_after_reload() {
        assert_eq!(
            BindingKey::from_logical(&Key::Shift),
            Some(BindingKey::Shift)
        );
        let mut bindings = Bindings::default();
        bindings.assign(Control::Ascend, BindingKey::Shift).unwrap();
        let mut restored: Bindings =
            serde_json::from_str(&serde_json::to_string(&bindings).unwrap()).unwrap();
        restored.validate();
        for key in [KeyCode::ShiftLeft, KeyCode::ShiftRight] {
            let mut physical = ButtonInput::default();
            physical.press(key);
            assert!(restored.pressed(Control::Ascend, &physical, &ButtonInput::default(), false));
            physical.release(key);
            assert!(!restored.pressed(Control::Ascend, &physical, &ButtonInput::default(), false));
        }
        let mut logical = ButtonInput::default();
        logical.press(Key::Character("e".into()));
        assert!(!restored.pressed(Control::Ascend, &ButtonInput::default(), &logical, false));
    }
    #[test]
    fn explicit_exchange_releases_aliases_and_preserves_unrelated_preferences() {
        let mut bindings = Bindings::default();
        bindings
            .assign(Control::Dock, BindingKey::Letter('h'))
            .unwrap();
        assert!(
            bindings
                .assign(Control::Forward, BindingKey::Letter('q'))
                .is_err()
        );
        bindings
            .swap(Control::Forward, BindingKey::Letter('q'))
            .unwrap();
        assert_eq!(bindings.keys(Control::Forward), &[BindingKey::Letter('q')]);
        assert_eq!(bindings.keys(Control::Left), &[BindingKey::Letter('z')]);
        assert_eq!(bindings.keys(Control::Dock), &[BindingKey::Letter('h')]);
        let mut restored: Bindings =
            serde_json::from_str(&serde_json::to_string(&bindings).unwrap()).unwrap();
        restored.validate();
        assert_eq!(restored.0, bindings.0);
    }
    #[test]
    fn exchange_rejects_third_context_conflict_without_partial_mutation() {
        let mut bindings = Bindings::default();
        // Jump and Ascend can share E, but Left and Jump cannot.
        bindings
            .assign(Control::Jump, BindingKey::Letter('e'))
            .unwrap();
        let before = serde_json::to_string(&bindings).unwrap();
        assert!(
            bindings
                .swap(Control::Ascend, BindingKey::Letter('q'))
                .is_err()
        );
        assert_eq!(serde_json::to_string(&bindings).unwrap(), before);
    }
}
