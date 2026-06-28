use std::time::Duration;

use bevy::prelude::*;

/// レース全体の設定。
/// Bevy 0.19では `Resource` が `Component` も実装するため、
/// `Component` を別途deriveしてはならない(二重実装エラーになる)。
#[derive(Resource)]
pub struct RaceConfig {
    pub lap_count: u32,
    pub track_length: f32,
}

impl Default for RaceConfig {
    fn default() -> Self {
        Self {
            lap_count: 1,
            track_length: 210.0,
        }
    }
}

/// `RacePhase::Countdown` 中に経過時間を計測するタイマー。
#[derive(Resource)]
pub struct CountdownTimer(pub Timer);

impl Default for CountdownTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(3.0, TimerMode::Once))
    }
}

/// レースタイマー。
#[derive(Resource, Default)]
pub struct RaceClock {
    pub elapsed: Duration,
    pub lap_times: Vec<Duration>,
}

/// 順位表。`DistanceAlongTrack` でソートされたエンティティ一覧。
#[derive(Resource, Default)]
pub struct Leaderboard(pub Vec<Entity>);
