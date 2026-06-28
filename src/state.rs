use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RacePhase {
    #[default]
    Countdown,
    Racing,
    Finished,
}
