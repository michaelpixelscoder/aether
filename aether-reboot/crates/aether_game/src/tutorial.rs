//! Advice advances only when the player performs the corresponding action.
use crate::{
    app::Phase,
    bindings::{Bindings, Control},
    persistence::{self, Preferences},
    session::GameSession,
};
use aether_sim::{Tether, Vessel};
use bevy::prelude::*;

pub const COMPLETE: u16 = 255;
#[derive(Clone, Copy)]
struct Sample {
    revision: u64,
    trim: f32,
    fuel: f32,
    docked: bool,
    tethered: bool,
    checkpoint: u32,
}
#[derive(Resource)]
pub struct Tutorial {
    previous: Option<Sample>,
    pub visible: bool,
}
impl Default for Tutorial {
    fn default() -> Self {
        Self {
            previous: None,
            visible: true,
        }
    }
}
impl Tutorial {
    pub fn reset_observation(&mut self) {
        self.previous = None;
    }
    fn observe(&mut self, current: Sample, mut flags: u16) -> u16 {
        if let Some(old) = self.previous {
            if current.revision != old.revision && current.docked {
                flags |= 1;
            }
            if (current.trim - old.trim).abs() > 0.0001 {
                flags |= 2;
            }
            if !current.docked {
                flags |= 4;
            }
            if !current.docked && current.fuel < old.fuel - 0.002 {
                flags |= 8;
            }
            if current.tethered {
                flags |= 16;
            }
            if old.tethered && !current.tethered {
                flags |= 32;
            }
            if current.docked && current.fuel > old.fuel + 0.002 {
                flags |= 64;
            }
            if current.checkpoint != 0 {
                flags |= 128;
            }
        }
        self.previous = Some(current);
        flags
    }
    pub fn advice(&self, flags: u16, keys: &Bindings) -> Option<String> {
        if !self.visible || flags == COMPLETE {
            return None;
        }
        let step = (0..8).find(|n| flags & (1 << n) == 0)?;
        let key = |c| keys.label(c);
        let instruction=match step {
            0=>format!("Au quai, ouvrez l'atelier ({}) et placez un bloc sur la coque. Ctrl+Z annule.",key(Control::Edit)),
            1=>format!("Quittez l'atelier ({}), larguez les amarres ({}), puis réglez la voile avec {} / {}.",key(Control::Edit),key(Control::Dock),key(Control::TrimLeft),key(Control::TrimRight)),
            2=>format!("Quittez l'atelier et larguez les amarres ({}). La sustentation maintient l'altitude choisie.",key(Control::Dock)),
            3=>"La sustentation dépense de l'Aether. Une coque plus lourde consomme davantage ; la voile utilise le vent gratuitement.".into(),
            4=>format!("Approchez une ancre dorée à moins de 70 m, sans obstacle, puis lancez le harpon ({}).",key(Control::Tether)),
            5=>format!("Le câble se tend et courbe votre trajectoire. Libérez-le ({}) pour conserver votre élan.",key(Control::Tether)),
            6=>format!("Pour recharger, freinez ({}) avant le quai, puis longez-le à son altitude. Amarrez-vous ({}) à moins de 9 m, sous 3,5 m/s.",key(Control::Backward),key(Control::Dock)),
            _=>format!("Choisissez une escale dans l'atlas ({}). Maintenez {} pour avancer ; le vent et les courants accélèrent le voyage. Le cap et l'altitude à droite vous guident.",key(Control::Atlas),key(Control::Forward)),
        };
        Some(format!(
            "APPRENTISSAGE · {}/8\n{}",
            flags.count_ones() + 1,
            instruction
        ))
    }
}
pub fn update(world: &mut World) {
    if !matches!(
        world.resource::<State<Phase>>().get(),
        Phase::Playing | Phase::Editing
    ) {
        return;
    }
    let game = world.resource::<GameSession>();
    let Some(v) = game.active.and_then(|e| world.get::<Vessel>(e)) else {
        return;
    };
    let mut sample = Sample {
        revision: v.body.revision(),
        trim: v.trim,
        fuel: v.fuel,
        docked: v.docked,
        tethered: false,
        checkpoint: game.checkpoint,
    };
    sample.tethered = world.query::<&Tether>().iter(world).next().is_some();
    let flags = world.resource::<Preferences>().tutorial;
    let next = world.resource_mut::<Tutorial>().observe(sample, flags);
    if next != flags {
        world.resource_mut::<Preferences>().tutorial = next;
        persistence::save_preferences(world);
    }
}
pub fn reset(world: &mut World) {
    world.insert_resource(Tutorial {
        visible: true,
        ..default()
    });
    world.resource_mut::<Preferences>().tutorial = 0;
    persistence::save_preferences(world);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tutorial_observes_real_changes_and_can_replay() {
        let mut t = Tutorial::default();
        let mut s = Sample {
            revision: 0,
            trim: 0.,
            fuel: 100.,
            docked: true,
            tethered: false,
            checkpoint: 0,
        };
        assert_eq!(t.observe(s, 0), 0);
        s.revision = 1;
        let mut flags = t.observe(s, 0);
        assert_eq!(flags, 1);
        s.docked = false;
        s.trim = 0.1;
        s.fuel = 99.;
        flags = t.observe(s, flags);
        assert_eq!(flags, 15);
        s.tethered = true;
        flags = t.observe(s, flags);
        assert_eq!(flags, 31);
        s.tethered = false;
        flags = t.observe(s, flags);
        assert_eq!(flags, 63);
        s.docked = true;
        s.fuel = 100.;
        s.checkpoint = 1;
        flags = t.observe(s, flags);
        assert_eq!(flags, COMPLETE);
        t = Tutorial::default();
        assert_eq!(t.observe(s, 0), 0);
    }
}
