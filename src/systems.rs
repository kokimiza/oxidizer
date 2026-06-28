use bevy::prelude::*;

use crate::components::*;
use crate::events::*;
use crate::resources::*;
use crate::state::RacePhase;

const ZONE_TRIGGER_RADIUS: f32 = 2.0;
const CHECKPOINT_TRIGGER_RADIUS: f32 = 3.0;
const RUST_PER_NEGATIVE_ELECTRON: f32 = 0.05;
const RECOVERY_PER_POSITIVE_ELECTRON: f32 = 0.02;
const BIKE_REDOX_RANGE: f32 = 3.0;
const BIKE_REDOX_RATE: f32 = 0.5;
const PLAYER_LATERAL_SPEED: f32 = 6.0;

/// プレイヤー操作中で腐食していないバイクのクエリ型(clippyのtype_complexity回避用)。
type PlayerInputQuery<'w, 's> =
    Query<'w, 's, (&'static mut Transform, &'static EffectiveSpeed, &'static SpeedStats), (With<PlayerControlled>, Without<Corroded>)>;

pub fn countdown_system(
    mut timer: ResMut<CountdownTimer>,
    time: Res<Time>,
    mut next_phase: ResMut<NextState<RacePhase>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        next_phase.set(RacePhase::Racing);
    }
}

/// プレイヤー入力を読み、前進(自動巡航+アクセル/ブレーキ)と左右の車線移動を行う。
pub fn input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: PlayerInputQuery,
    time: Res<Time>,
) {
    for (mut transform, effective, stats) in &mut query {
        let mut throttle = 0.6;
        if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
            throttle = 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
            throttle = 0.2;
        }

        let mut lateral = 0.0;
        if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
            lateral -= 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
            lateral += 1.0;
        }

        transform.translation.x += effective.0 * throttle * time.delta_secs();
        transform.translation.z += lateral * PLAYER_LATERAL_SPEED * stats.handling * time.delta_secs();
    }
}

/// 敵バイクはレーンを保ったまま自動で前進する。
pub fn ai_movement_system(
    mut query: Query<(&mut Transform, &EffectiveSpeed, &AiControlled), Without<Corroded>>,
    time: Res<Time>,
) {
    for (mut transform, effective, ai) in &mut query {
        transform.translation.x += effective.0 * ai.difficulty * time.delta_secs();
    }
}

pub fn movement_system(mut query: Query<(&Transform, &mut DistanceAlongTrack), With<Bike>>) {
    for (transform, mut distance) in &mut query {
        distance.0 = transform.translation.x;
    }
}

pub fn zone_detection_system(
    bikes: Query<(Entity, &Transform), With<Bike>>,
    reductant_pads: Query<(&Transform, &ReductantPad)>,
    oxidant_hazards: Query<(&Transform, &OxidantHazard)>,
    mut transfer_events: MessageWriter<ElectronTransferEvent>,
) {
    for (bike, bike_transform) in &bikes {
        for (pad_transform, pad) in &reductant_pads {
            if bike_transform.translation.distance(pad_transform.translation) <= ZONE_TRIGGER_RADIUS
            {
                transfer_events.write(ElectronTransferEvent {
                    bike,
                    delta: pad.electron_gain,
                });
            }
        }
        for (hazard_transform, hazard) in &oxidant_hazards {
            if bike_transform
                .translation
                .distance(hazard_transform.translation)
                <= ZONE_TRIGGER_RADIUS
            {
                transfer_events.write(ElectronTransferEvent {
                    bike,
                    delta: -hazard.electron_loss,
                });
            }
        }
    }
}

/// 敵バイクと接近すると、電位差(電子数の差)に応じて電子が多い側から少ない側へ移動する。
/// 環境ゾーンとは異なり、移動量の合計は常に0(一方の酸化が他方の還元を生む、本来のredox)。
pub fn bike_redox_exchange_system(
    bikes: Query<(Entity, &Transform, &ElectronCount), With<Bike>>,
    mut transfer_events: MessageWriter<ElectronTransferEvent>,
    time: Res<Time>,
) {
    for [(bike_a, transform_a, electrons_a), (bike_b, transform_b, electrons_b)] in
        bikes.iter_combinations()
    {
        if transform_a.translation.distance(transform_b.translation) > BIKE_REDOX_RANGE {
            continue;
        }

        let potential_diff = (electrons_a.0 - electrons_b.0) as f32;
        let flow = (potential_diff * BIKE_REDOX_RATE * time.delta_secs()) as i32;
        if flow == 0 {
            continue;
        }

        transfer_events.write(ElectronTransferEvent {
            bike: bike_a,
            delta: -flow,
        });
        transfer_events.write(ElectronTransferEvent {
            bike: bike_b,
            delta: flow,
        });
    }
}

pub fn electron_transfer_system(
    mut query: Query<&mut ElectronCount>,
    mut transfer_events: MessageReader<ElectronTransferEvent>,
) {
    for event in transfer_events.read() {
        if let Ok(mut electrons) = query.get_mut(event.bike) {
            electrons.0 += event.delta;
        }
    }
}

pub fn rust_and_integrity_system(
    mut query: Query<(&ElectronCount, &mut RustCoating, &mut Integrity)>,
) {
    for (electrons, mut rust, mut integrity) in &mut query {
        if electrons.0 < 0 {
            let increase = -electrons.0 as f32 * RUST_PER_NEGATIVE_ELECTRON;
            rust.0 += increase;
            integrity.0 = (integrity.0 - increase).max(0.0);
        } else if electrons.0 > 0 {
            let recovery = electrons.0 as f32 * RECOVERY_PER_POSITIVE_ELECTRON;
            integrity.0 = (integrity.0 + recovery).min(100.0);
        }
    }
}

pub fn corrosion_system(
    mut commands: Commands,
    time: Res<Time>,
    mut newly_corroded: Query<(Entity, &Integrity), Without<Corroded>>,
    mut recovering: Query<(Entity, &mut Corroded, &mut Integrity)>,
    mut corrosion_events: MessageWriter<CorrosionEvent>,
) {
    for (bike, integrity) in &mut newly_corroded {
        if integrity.0 <= 0.0 {
            commands.entity(bike).insert(Corroded {
                recovery_timer: Timer::from_seconds(3.0, TimerMode::Once),
            });
            corrosion_events.write(CorrosionEvent { bike });
        }
    }

    for (bike, mut corroded, mut integrity) in &mut recovering {
        corroded.recovery_timer.tick(time.delta());
        if corroded.recovery_timer.is_finished() {
            integrity.0 = 25.0;
            commands.entity(bike).remove::<Corroded>();
        }
    }
}

/// 錆の蓄積量から、そのフレームの実効速度を `SpeedStats::base_speed` を基準に再計算する。
/// `base_speed` 自体は変更しないため、錆が減れば速度はそのフレームで回復する。
pub fn speed_modifier_system(mut query: Query<(&SpeedStats, &RustCoating, &mut EffectiveSpeed)>) {
    for (stats, rust, mut effective) in &mut query {
        let penalty = (rust.0 * 0.01).min(0.9);
        effective.0 = stats.base_speed * (1.0 - penalty);
    }
}

pub fn checkpoint_and_lap_system(
    mut bikes: Query<(Entity, &Transform, &mut LapProgress)>,
    checkpoints: Query<(&Transform, &Checkpoint)>,
    finish_lines: Query<&Transform, With<FinishLine>>,
    race_clock: Res<RaceClock>,
    mut checkpoint_events: MessageWriter<CheckpointPassedEvent>,
    mut lap_events: MessageWriter<LapCompletedEvent>,
) {
    for (bike, transform, mut progress) in &mut bikes {
        for (checkpoint_transform, checkpoint) in &checkpoints {
            if (transform.translation.x - checkpoint_transform.translation.x).abs()
                <= CHECKPOINT_TRIGGER_RADIUS
                && progress.checkpoints_hit.insert(checkpoint.index)
            {
                checkpoint_events.write(CheckpointPassedEvent {
                    bike,
                    index: checkpoint.index,
                });
            }
        }

        for finish_transform in &finish_lines {
            if (transform.translation.x - finish_transform.translation.x).abs()
                <= CHECKPOINT_TRIGGER_RADIUS
                && !progress.checkpoints_hit.is_empty()
            {
                progress.current_lap += 1;
                progress.checkpoints_hit.clear();
                lap_events.write(LapCompletedEvent {
                    bike,
                    lap: progress.current_lap,
                    time: race_clock.elapsed,
                });
            }
        }
    }
}

pub fn ranking_system(
    bikes: Query<(Entity, &DistanceAlongTrack), With<Bike>>,
    mut leaderboard: ResMut<Leaderboard>,
) {
    let mut ranked: Vec<(Entity, f32)> = bikes
        .iter()
        .map(|(entity, distance)| (entity, distance.0))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    leaderboard.0 = ranked.into_iter().map(|(entity, _)| entity).collect();
}

/// 誰か1台でも `lap_count` に到達したらレース終了(簡易な早着判定)。
pub fn race_finish_system(
    config: Res<RaceConfig>,
    bikes: Query<&LapProgress, With<Bike>>,
    leaderboard: Res<Leaderboard>,
    mut next_phase: ResMut<NextState<RacePhase>>,
    mut race_finished_events: MessageWriter<RaceFinishedEvent>,
) {
    let any_finished = bikes
        .iter()
        .any(|progress| progress.current_lap >= config.lap_count);

    if any_finished {
        next_phase.set(RacePhase::Finished);
        race_finished_events.write(RaceFinishedEvent {
            ranking: leaderboard.0.clone(),
        });
    }
}

pub fn race_clock_system(mut clock: ResMut<RaceClock>, time: Res<Time>) {
    clock.elapsed += time.delta();
}

pub fn corrosion_feedback_system(mut events: MessageReader<CorrosionEvent>) {
    for event in events.read() {
        warn!("Bike {:?} corroded and is temporarily disabled", event.bike);
    }
}

pub fn checkpoint_feedback_system(mut events: MessageReader<CheckpointPassedEvent>) {
    for event in events.read() {
        info!("Bike {:?} passed checkpoint {}", event.bike, event.index);
    }
}

/// ラップ完了をログ出力し、`RaceClock::lap_times` に記録する。
pub fn lap_feedback_system(
    mut events: MessageReader<LapCompletedEvent>,
    mut clock: ResMut<RaceClock>,
) {
    for event in events.read() {
        info!(
            "Bike {:?} completed lap {} at {:.2}s",
            event.bike,
            event.lap,
            event.time.as_secs_f32()
        );
        clock.lap_times.push(event.time);
    }
}

pub fn race_finished_feedback_system(mut events: MessageReader<RaceFinishedEvent>) {
    for event in events.read() {
        info!("Race finished! Ranking: {:?}", event.ranking);
    }
}

pub fn camera_follow_system(
    player: Query<&Transform, (With<PlayerControlled>, Without<Camera3d>)>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    camera_transform.translation = player_transform.translation + Vec3::new(-12.0, 6.0, 0.0);
    let look_target = player_transform.translation + Vec3::new(5.0, 0.0, 0.0);
    camera_transform.look_at(look_target, Vec3::Y);
}

pub fn hud_update_system(
    player: Query<(&ElectronCount, &Integrity, &RustCoating, &LapProgress), With<PlayerControlled>>,
    phase: Res<State<RacePhase>>,
    countdown: Res<CountdownTimer>,
    config: Res<RaceConfig>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let Ok((electrons, integrity, rust, progress)) = player.single() else {
        return;
    };
    let Ok(mut text) = hud.single_mut() else {
        return;
    };

    let phase_line = match phase.get() {
        RacePhase::Countdown => {
            let remaining = (countdown.0.duration() - countdown.0.elapsed()).as_secs_f32();
            format!("START IN {:.1}", remaining.max(0.0))
        }
        RacePhase::Racing => "RACING".to_string(),
        RacePhase::Finished => "FINISH!".to_string(),
    };

    *text = Text::new(format!(
        "{phase_line}\nLap {}/{}\nElectrons: {}\nIntegrity: {:.0}\nRust: {:.0}",
        progress.current_lap, config.lap_count, electrons.0, integrity.0, rust.0
    ));
}
