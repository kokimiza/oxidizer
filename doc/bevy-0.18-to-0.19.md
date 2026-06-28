# Bevy 0.18 → 0.19 変更点まとめ

このプロジェクトは `bevy = "0.19.0"` を使用している。0.18からの主要な変更点を整理する。

参考:
- [公式マイグレーションガイド](https://bevy.org/learn/migration-guides/0-18-to-0-19/)
- [Bevy 0.19 リリースノート](https://bevy.org/news/bevy-0-19/)

## 主要な新機能

- **Next Generation Scenes (BSN)**: `bsn!` マクロによるコード内シーン記述。コンポーザブルなパッチ、オプショナルフィールド、リレーションシップ、シーン関数、アセット依存をサポート。
- **大規模シーンの描画高速化**: GPUバッチアンパック、スパースメッシュアップロード、ライトのGPUクラスタリング、バッチ化されたモーフターゲットなどにより、160万キューブで21FPS→53FPSに改善。
- **コンタクトシャドウ**: シャドウマップを補完するスクリーンスペースのコンタクトシャドウ。
- **テキスト入力 (`EditableText`)**: カーソル移動、選択、クリップボード、Unicode対応、双方向テキスト、IME対応。
- **リッチテキスト**: ジェネリックフォントファミリー(Serif, Monospaceなど)、可変フォント(weight/width/style)、Vh/Remなどのレスポンシブ単位、文字間隔調整。
- **物理ベーススクリーンスペースリフレクション**
- **矩形エリアライト**: Linearly Transformed Cosinesによるリアルタイム照明(シャドウ未対応)。
- **Feathersウィジェット追加**: テキスト入力、数値入力、ドロップダウン、ディスクロージャートグル、リストビュー、スクロールバー(すべてBSNベース)。
- **App Settings**: ECSリソースとしての設定の読み込み/保存と自動永続化。
- **ポストプロセス追加**: ビネット、レンズディストーション。
- **スキンメッシュのカリング改善**: 実際のジョイント位置からバウンディングを計算し、アニメーション中の肢の消失を防止。
- **レンダー復旧**: GPUエラーが型付き例外として表面化し、再初期化や安全なシャットダウンが可能に。

## 破壊的変更(Migration Guide)

### Resourcesがコンポーネント化
- `#[derive(Resource)]` が `Component` を実装するようになった。`Component` と `Resource` の二重derive不可。
- `ReflectResource` はZSTになったため `ReflectComponent` を使用。
- `Query<()>` などの広いクエリがリソースアクセスと衝突する可能性。
- non-send resource関連メソッドが `non_send` 系にリネーム(例: `init_non_send_resource` → `init_non_send`)。
- `World::clear_entities` がリソースもクリアするようになった。
- `World::remove_resource_by_id` の戻り値が `bool` に変更。

### テキスト/フォント (Cosmic Text → Parley)
- `Font::try_from_bytes` → `Font::from_bytes`(`Result` を返さなくなった)。
- `TextRoot`/`TextSpanAccess`/`TextSpanComponent` が `TextSection` トレイトに統合。
- `TextFont::font` が `Handle<Font>` から `FontSource` に変更。
- `TextFont::font_size` が `f32` から `FontSize` に変更。
- `TextLayout::new_with_justify` → `justify`、`new_with_linebreak` → `linebreak`、`new_with_no_wrap` → `no_wrap`。
- `PositionedGlyph::span_index` → `section_index`。`byte_index`/`byte_length` フィールド削除。

### シーン → ワールドシリアライゼーション
- `bevy_scene` クレートが `bevy_world_serialization` にリネーム。
- `Scene` → `WorldAsset`、`SceneRoot` → `WorldAssetRoot`、`DynamicScene` → `DynamicWorld`、`SceneSpawner` → `WorldInstanceSpawner` など、"Scene" 用語が全面的に "WorldSerialization" 系へ改名。
- `DynamicWorldBuilder` などが `&TypeRegistry` を要求するようになった。

### Cargoフィーチャー
- `experimental_bevy_feathers` → `bevy_feathers`、`experimental_ui_widgets` → `bevy_ui_widgets`。
- `audio` フィーチャーが `3d`/`2d`/`ui` から暗黙的に有効化されなくなった(明示的指定が必要)。
- Rodio 0.22対応でオーディオフォーマットフィーチャーが再編(`vorbis`, `wav`, `mp3`, `mp4`, `flac`, `aac`, `audio-all-formats` など)。

### レンダリング/マテリアル
- 新クレート `bevy_material` が追加され、`bevy_pbr`/`bevy_render` からマテリアル関連型が移動。
- glTFマテリアルロードが `StandardMaterial` ではなく `GltfMaterial` を返すようになった。`StandardMaterial` をロードするには `/std` サフィックスが必要。
- レンダーグラフがシステムベースのアーキテクチャに変換。
- `RenderSystems::ManageViews` が `CreateViews`/`Specialize`/`PrepareViews` の3フェーズに分割。
- `Skybox::image` が `Handle<Image>` から `Option<Handle<Image>>` に変更(既存コードは `Some()` でラップが必要)。
- `Atmosphere` がカメラコンポーネントからエンティティに変更。`bevy::pbr::Atmosphere` → `bevy::light::Atmosphere`。
- Bloomの輝度計算がリニア空間で行われるようになり、見た目がわずかに変化する場合がある。

### UI/入力
- `UiWidgetsPlugins`/`InputDispatchPlugin` が `DefaultPlugins` に統合。
- `InputFocus` のフィールドが非公開になり、`get()`/`set()`/`clear()` メソッド経由でアクセス。
- UIウィジェットコンポーネントの `Core` プレフィックスが削除(例: `CoreScrollbarThumb` → `ScrollbarThumb`)。

### システム実行
- `ExecutorKind` が削除され、`Schedule::set_executor` で実行器インスタンス(`SingleThreadedExecutor::new()` など)を直接指定する方式に変更。
- `System::type_id` → `System::system_type`。
- `SystemParam` の検証がデータ取得時に行われるようになった(カスタム実装の更新が必要)。

### イベント → メッセージ
- 公式マイグレーションガイドのページに明記されていないが、実際にビルドして判明した変更。バッファ型のイベントAPIが全面的にリネームされている。
  - `#[derive(Event)]` → `#[derive(Message)]`
  - `EventReader<T>` → `MessageReader<T>`、`EventWriter<T>` → `MessageWriter<T>`
  - `App::add_event::<T>()` → `App::add_message::<T>()`
  - `Event` トレイト自体は残るが、observer(`On<T>`)で使うentity-event専用になった。バッファして毎フレーム`read()`するタイプのイベントは `Message` 系を使う。

### Timer
- `Timer::finished()` → `Timer::is_finished()`(同様にマイグレーションガイドのページには記載なし)。

### その他
- `rand` の `RngCore` → `Rng`、`Rng` → `RngExt` にリネーム。
- `DefaultErrorHandler` → `FallbackErrorHandler`。
- ライトギズモが `bevy_gizmos` から `bevy_light` に移動。
- `Task<T>` のドロップがWebビルドでキャンセルを引き起こすようになった(従来の挙動を維持するには `detach()` を呼ぶ)。

## このプロジェクトへの影響

[main.rs](../src/main.rs) は現時点でBevyの基本機能のみを使用しているため、上記破壊的変更の直接的な影響は限定的と見られる。今後 Resource/Scene/Text/Audio 関連のAPIを使う際は本ガイドを参照すること。

実際に[酸化還元レースゲームのECS設計](ecs-design-redox-racing.md)を実装した際、上記の「イベント → メッセージ」「Timer」の変更はマイグレーションガイドのページに記載がなく、`cargo check` のエラーから発見した。公式ガイドだけに頼らず、ビルドして確認することが重要。
