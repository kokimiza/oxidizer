mod components;
mod events;
mod plugin;
mod resources;
mod state;
mod systems;

use bevy::prelude::*;
use components::*;
use plugin::RacePlugin;
use resources::RaceConfig;

const START_X: f32 = 0.0;
const LANES: [f32; 8] = [-7.0, -5.0, -3.0, -1.0, 1.0, 3.0, 5.0, 7.0];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RacePlugin)
        .add_systems(Startup, (spawn_scene, spawn_track_and_bikes, spawn_hud))
        .run();
}

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<RaceConfig>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-12.0, 6.0, 0.0).looking_at(Vec3::new(START_X + 5.0, 0.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(-0.5, -1.0, -0.3), Vec3::Y),
    ));

    let track_mesh = meshes.add(Plane3d::default().mesh().size(config.track_length + 20.0, 20.0));
    let track_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.25, 0.28),
        ..default()
    });
    commands.spawn((
        Mesh3d(track_mesh),
        MeshMaterial3d(track_material),
        Transform::from_xyz(config.track_length / 2.0, -0.05, 0.0),
    ));
}

fn spawn_track_and_bikes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<RaceConfig>,
) {
    let checkpoint_x = config.track_length * 100.0 / 210.0;
    let finish_x = config.track_length;
    let bike_mesh = meshes.add(Cuboid::new(1.6, 0.6, 0.4));

    let player_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.4, 1.0),
        ..default()
    });
    commands.spawn((
        Bike,
        PlayerControlled,
        ElectronCount(0),
        Integrity::default(),
        RustCoating::default(),
        EffectiveSpeed::default(),
        SpeedStats {
            base_speed: 14.0,
            handling: 1.0,
        },
        DistanceAlongTrack::default(),
        LapProgress::default(),
        Mesh3d(bike_mesh.clone()),
        MeshMaterial3d(player_material),
        Transform::from_xyz(START_X, 0.3, LANES[0]),
    ));

    for (i, &lane) in LANES[1..].iter().enumerate() {
        let hue = (i as f32) / 7.0 * 360.0;
        let ai_material = materials.add(StandardMaterial {
            base_color: Color::hsl(hue, 0.8, 0.5),
            ..default()
        });
        commands.spawn((
            Bike,
            AiControlled {
                difficulty: 0.75 + 0.05 * i as f32,
            },
            ElectronCount((i as i32 - 3) * 4),
            Integrity::default(),
            RustCoating::default(),
            EffectiveSpeed::default(),
            SpeedStats {
                base_speed: 12.0 + i as f32 * 0.3,
                handling: 1.0,
            },
            DistanceAlongTrack::default(),
            LapProgress::default(),
            Mesh3d(bike_mesh.clone()),
            MeshMaterial3d(ai_material),
            Transform::from_xyz(START_X, 0.3, lane),
        ));
    }

    let pad_mesh = meshes.add(Sphere::new(1.0));
    let pad_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 1.0, 0.4),
        emissive: LinearRgba::new(0.0, 3.0, 1.0, 1.0),
        ..default()
    });
    for x in [40.0, 150.0] {
        commands.spawn((
            ReductantPad { electron_gain: 8 },
            Mesh3d(pad_mesh.clone()),
            MeshMaterial3d(pad_material.clone()),
            Transform::from_xyz(x, 0.5, 0.0),
        ));
    }

    let hazard_mesh = meshes.add(Sphere::new(1.0));
    let hazard_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.35, 0.05),
        emissive: LinearRgba::new(3.0, 1.0, 0.0, 1.0),
        ..default()
    });
    commands.spawn((
        OxidantHazard { electron_loss: 8 },
        Mesh3d(hazard_mesh),
        MeshMaterial3d(hazard_material),
        Transform::from_xyz(80.0, 0.5, 0.0),
    ));

    let gate_mesh = meshes.add(Cuboid::new(0.5, 4.0, 16.0));
    let checkpoint_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 0.2, 0.4),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Checkpoint { index: 0 },
        Mesh3d(gate_mesh.clone()),
        MeshMaterial3d(checkpoint_material),
        Transform::from_xyz(checkpoint_x, 2.0, 0.0),
    ));

    let finish_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.6),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        FinishLine,
        Mesh3d(gate_mesh),
        MeshMaterial3d(finish_material),
        Transform::from_xyz(finish_x, 2.0, 0.0),
    ));
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Text::new("START IN 3.0"),
        TextFont {
            font_size: 28.0.into(),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
        HudText,
    ));
}
