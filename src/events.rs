use std::time::Duration;

use bevy::prelude::*;

// Bevy 0.19ではバッファ型のイベントは `Event`/`EventReader`/`EventWriter` から
// `Message`/`MessageReader`/`MessageWriter` に名称統合された
// (`Event` はオブザーバー向けのエンティティイベント専用になった)。

/// 環境ゾーン通過、または対バイクの電子交換で発火し、電子数の変動を伝える。
#[derive(Message)]
pub struct ElectronTransferEvent {
    pub bike: Entity,
    pub delta: i32,
}

/// `Integrity` が0以下になったときに発火。
#[derive(Message)]
pub struct CorrosionEvent {
    pub bike: Entity,
}

/// チェックポイント通過時に発火。
#[derive(Message)]
pub struct CheckpointPassedEvent {
    pub bike: Entity,
    pub index: u32,
}

/// ラップ完了時に発火。
#[derive(Message)]
pub struct LapCompletedEvent {
    pub bike: Entity,
    pub lap: u32,
    pub time: Duration,
}

/// レース終了時に発火。
#[derive(Message)]
pub struct RaceFinishedEvent {
    pub ranking: Vec<Entity>,
}
