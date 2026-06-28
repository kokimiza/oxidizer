use bevy::prelude::*;

use crate::events::*;
use crate::resources::*;
use crate::state::RacePhase;
use crate::systems::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RaceSet {
    Input,
    Movement,
    ZoneDetection,
    ElectronTransfer,
    RustAndIntegrity,
    Corrosion,
    SpeedModifier,
    CheckpointAndLap,
    Ranking,
    Finish,
}

pub struct RacePlugin;

impl Plugin for RacePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<RacePhase>()
            .init_resource::<RaceConfig>()
            .init_resource::<RaceClock>()
            .init_resource::<Leaderboard>()
            .init_resource::<CountdownTimer>()
            .add_message::<ElectronTransferEvent>()
            .add_message::<CorrosionEvent>()
            .add_message::<CheckpointPassedEvent>()
            .add_message::<LapCompletedEvent>()
            .add_message::<RaceFinishedEvent>()
            .configure_sets(
                Update,
                (
                    RaceSet::Input,
                    RaceSet::Movement,
                    RaceSet::ZoneDetection,
                    RaceSet::ElectronTransfer,
                    RaceSet::RustAndIntegrity,
                    RaceSet::Corrosion,
                    RaceSet::SpeedModifier,
                    RaceSet::CheckpointAndLap,
                    RaceSet::Ranking,
                    RaceSet::Finish,
                )
                    .chain()
                    .run_if(in_state(RacePhase::Racing)),
            )
            .add_systems(Update, race_clock_system.run_if(in_state(RacePhase::Racing)))
            .add_systems(
                Update,
                countdown_system.run_if(in_state(RacePhase::Countdown)),
            )
            .add_systems(Update, (camera_follow_system, hud_update_system))
            .add_systems(Update, input_system.in_set(RaceSet::Input))
            .add_systems(Update, ai_movement_system.in_set(RaceSet::Input))
            .add_systems(Update, movement_system.in_set(RaceSet::Movement))
            .add_systems(Update, zone_detection_system.in_set(RaceSet::ZoneDetection))
            .add_systems(
                Update,
                bike_redox_exchange_system.in_set(RaceSet::ZoneDetection),
            )
            .add_systems(
                Update,
                electron_transfer_system.in_set(RaceSet::ElectronTransfer),
            )
            .add_systems(
                Update,
                rust_and_integrity_system.in_set(RaceSet::RustAndIntegrity),
            )
            .add_systems(Update, corrosion_system.in_set(RaceSet::Corrosion))
            .add_systems(
                Update,
                speed_modifier_system.in_set(RaceSet::SpeedModifier),
            )
            .add_systems(
                Update,
                checkpoint_and_lap_system.in_set(RaceSet::CheckpointAndLap),
            )
            .add_systems(Update, ranking_system.in_set(RaceSet::Ranking))
            .add_systems(Update, race_finish_system.in_set(RaceSet::Finish));
    }
}
