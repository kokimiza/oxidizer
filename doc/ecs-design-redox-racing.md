# 酸化還元レースゲーム ECS設計 (Bevy 0.19)

[Bevy 0.18→0.19の変更点](bevy-0.18-to-0.19.md) を踏まえたECS設計。

## コンセプト

全車両は**バイク(2輪車)**。バイク同士が電子の授受(酸化還元反応)で性能が変化するレースゲーム。

- 各バイクは「電子数 (ElectronCount)」を持つ。
- **敵バイクと接近・接触すると、電子のやり取り(酸化還元反応)が発生する。** 電子の多いバイクから少ないバイクへ電子が流れ、合計電子数は両者間で保存される(一方が酸化されれば他方が還元される、という本来のredoxに忠実な挙動)。
  - 電子を奪われた(酸化された)バイクは速度が落ち、錆が蓄積する。
  - 電子を受け取った(還元された)バイクは加速力が上がる。
  - そのため、電子の多い敵の近くにとどまって電子を吸い取るか、自分が電子持ちのときは敵から距離を取って守るか、というポジショニングの駆け引きが生まれる。
- コース上の還元/酸化ゾーン(ブーストパッド/ハザード)は環境要因として残し、外部から電子を補充・剥奪する役割を持つ。
- 錆が一定量を超えると **腐食 (Corrosion)** が発生し、一定時間操作不能になる(DNFリスク)。
- 電子数のバランス(還元電位)を管理しながらラップを重ねて勝敗を競う。

## エンティティ構成

| エンティティ種別 | 主なコンポーネント |
|---|---|
| バイク | `Bike`, `ElectronCount`, `Integrity`, `RustCoating`, `SpeedStats`, `LapProgress`, `DistanceAlongTrack`, `PlayerControlled` または `AiControlled` |
| 還元ゾーン(ブーストパッド) | `ReductantPad`, `Transform`, `Collider` |
| 酸化ゾーン(ハザード) | `OxidantHazard`, `Transform`, `Collider` |
| チェックポイント | `Checkpoint`, `Transform` |
| フィニッシュライン | `FinishLine` |
| レース管理用エンティティ | `RaceConfig`, `RaceClock`, `Leaderboard` (0.19よりResourceもコンポーネントとして専用エンティティに格納される) |

## コンポーネント設計

```text
Bike                          // マーカー。全車両はこれのみ(車種区分なし)
ElectronCount(i32)            // 現在の電子数。正なら還元優勢、負なら酸化優勢
Integrity(f32)                // 車体強度 0.0-100.0。0で腐食(corrosion)
RustCoating(f32)               // 錆の蓄積量。速度低下の係数に直結
SpeedStats { base_speed, accel, handling }
DistanceAlongTrack(f32)       // 順位計算用のコース上距離
LapProgress { current_lap, checkpoints_hit: HashSet<u32> }
PlayerControlled              // マーカー
AiControlled { difficulty }   // 敵バイク
ReductantPad { electron_gain: i32 }
OxidantHazard { electron_loss: i32 }
Checkpoint { index: u32 }
FinishLine
Corroded { recovery_timer: Timer }  // 腐食中、操作を無効化するマーカー
```

`Resource` は0.19で `Component` を実装するため、`RaceConfig` 等のグローバル状態を不注意に `Query<()>` 等の広いクエリで巻き込まないよう、専用の `RaceConfig` 等にも `#[derive(Resource)]` のみを付与し `Component` を二重derivしない([破壊的変更]参照)。

## リソース(0.19: 専用エンティティ上のコンポーネントとして実装)

```text
RaceConfig { lap_count: u32, track_length: f32 }
RaceClock { elapsed: Duration, lap_times: Vec<Duration> }
Leaderboard(Vec<Entity>)  // DistanceAlongTrackでソートされた順位
```

## 状態 (States)

```text
enum RacePhase {
    Countdown,
    Racing,
    Finished,
}
```

`OnEnter(RacePhase::Racing)` でタイマー開始、`OnEnter(RacePhase::Finished)` で `Leaderboard` を確定する。

## イベント

```text
ElectronTransferEvent { bike: Entity, delta: i32 }          // パッド/ハザード通過、または対バイク電子交換で発火
CorrosionEvent { bike: Entity }                              // Integrity <= 0
CheckpointPassedEvent { bike: Entity, index: u32 }
LapCompletedEvent { bike: Entity, lap: u32, time: Duration }
RaceFinishedEvent { ranking: Vec<Entity> }
```

## バイク対バイクの電子交換(中核メカニクス)

`bike_redox_exchange_system` が `Bike` の全ペアを総当たりし(`Query::iter_combinations::<2>`)、`BIKE_REDOX_RANGE` 以内に近づいたペアについて以下を行う:

1. 2台の `ElectronCount` の差分(電位差)を求める。
2. 差分に比例した量の電子を、電子が多い側から少ない側へ毎フレーム移動させる(`BIKE_REDOX_RATE` で速度を調整)。
3. 片方に `-delta`、もう片方に `+delta` の `ElectronTransferEvent` を発火する(合計は常に0 — 環境ゾーンとは異なり、対バイク間では電子は生成・消滅しない)。

この移動量は `electron_transfer_system` で既存の `ElectronCount` に適用されるため、パッド/ハザードからの電子授受と完全に同じパイプラインに乗る。結果として:

- 電子を多く持つ敵に張り付く(ドラフティング)と、自分が還元されて加速力が上がる代わりに、相手は酸化されて錆びていく。
- 自分が電子持ちの状態で敵に追われると、逆に吸い取られるリスクがある。

## システム(実行順)

`Update` スケジュール内、`SystemSet` で順序を固定:

1. `input_system` — `PlayerControlled` の入力を `SpeedStats` に反映(`Corroded` 中はスキップ)
2. `movement_system` — `Transform` を前進させ `DistanceAlongTrack` を更新
3. `zone_detection_system` — `ReductantPad`/`OxidantHazard` との衝突判定 → `ElectronTransferEvent` 発火
3'. `bike_redox_exchange_system` — バイク同士の接近判定 → 電位差に応じた `ElectronTransferEvent` を双方に発火(3と同じ`SystemSet`内、順不同で並行)
4. `electron_transfer_system` — イベントを受けて `ElectronCount` を更新
5. `rust_and_integrity_system` — `ElectronCount` が負側に振れるほど `RustCoating` 増加・`Integrity` 減少。正側なら緩やかに回復
6. `corrosion_system` — `Integrity <= 0.0` で `Corroded` を付与、`CorrosionEvent` 発火。タイマー終了で `Corroded` 除去・`Integrity` 一部回復
7. `speed_modifier_system` — `RustCoating`/`ElectronCount` から実効最大速度を再計算
8. `checkpoint_and_lap_system` — `Checkpoint`/`FinishLine` 通過判定、`LapProgress` 更新、各種イベント発火
9. `ranking_system` — `DistanceAlongTrack` でソートし `Leaderboard` を更新
10. `race_finish_system` — `RacePhase::Racing` 中に全バイクが `lap_count` を満たしたら `RacePhase::Finished` へ遷移

UI/HUD系は0.19のリッチテキスト(`FontSize`, `FontSource`)とテキスト入力(`EditableText`)を利用し、`Countdown` 画面でのプレイヤー名入力や、レース中の電子数/錆ゲージ表示に使う。

## モジュール構成 (src/)

```text
src/
  main.rs        // App構築、DefaultPlugins + RacePlugin登録
  state.rs       // RacePhase
  components.rs  // バイク・コース系コンポーネント
  resources.rs   // RaceConfig / RaceClock / Leaderboard
  events.rs      // 上記イベント群
  plugin.rs      // RacePlugin: システムの登録とスケジューリング
  systems.rs     // 上記システム群の実装
```
