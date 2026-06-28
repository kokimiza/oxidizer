use std::collections::HashSet;

use bevy::prelude::*;

/// バイク(2輪車)を表すマーカー。全車両はバイクのみ。
#[derive(Component)]
pub struct Bike;

/// 現在の電子数。正なら還元優勢、負なら酸化優勢。
#[derive(Component, Default)]
pub struct ElectronCount(pub i32);

/// 車体強度。0以下になると腐食する。
#[derive(Component)]
pub struct Integrity(pub f32);

impl Default for Integrity {
    fn default() -> Self {
        Self(100.0)
    }
}

/// 錆の蓄積量。速度低下の係数に直結する。
#[derive(Component, Default)]
pub struct RustCoating(pub f32);

/// 車両の基礎性能。
#[derive(Component)]
pub struct SpeedStats {
    pub base_speed: f32,
    pub handling: f32,
}

/// コース上の進行距離。順位計算に使う。
#[derive(Component, Default)]
pub struct DistanceAlongTrack(pub f32);

/// 錆の影響を反映した、現在フレームの実効前進速度。
#[derive(Component, Default)]
pub struct EffectiveSpeed(pub f32);

/// HUDテキストを示すマーカー。
#[derive(Component)]
pub struct HudText;

/// ラップ進行状況。
#[derive(Component, Default)]
pub struct LapProgress {
    pub current_lap: u32,
    pub checkpoints_hit: HashSet<u32>,
}

/// プレイヤー操作の車両であることを示すマーカー。
#[derive(Component)]
pub struct PlayerControlled;

/// AI操作の車両であることを示す。
#[derive(Component)]
pub struct AiControlled {
    pub difficulty: f32,
}

/// 還元ゾーン(ブーストパッド)。通過すると電子を得る。
#[derive(Component)]
pub struct ReductantPad {
    pub electron_gain: i32,
}

/// 酸化ゾーン(ハザード)。通過すると電子を失う。
#[derive(Component)]
pub struct OxidantHazard {
    pub electron_loss: i32,
}

/// チェックポイント。
#[derive(Component)]
pub struct Checkpoint {
    pub index: u32,
}

/// フィニッシュラインを示すマーカー。
#[derive(Component)]
pub struct FinishLine;

/// 腐食状態。操作不能になっている間付与される。
#[derive(Component)]
pub struct Corroded {
    pub recovery_timer: Timer,
}
