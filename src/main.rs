#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::window::{PresentMode, WindowTheme};

mod audio_capture;
mod audio_processing;
mod bar_material;
mod cfg;
mod circle_material;
mod circle_split_material;
mod polygon_material;
mod visualization;

use crate::audio_capture::{audio_capture_startup_system, AudioReceiver};
use crate::audio_processing::{audio_event_system, AudioVisualizerState};
use crate::bar_material::{AudioEntity, AudioMaterial};
use crate::cfg::*;
use crate::circle_material::{CircleEntity, CircleMaterial};
use crate::circle_split_material::{CircleSplitEntity, CircleSplitMaterial};
use crate::polygon_material::{PolygonEntity, PolygonMaterial};
use crate::visualization::{
    spawn_visualization, visualization_toggle_system, window_resized_event, VisualizationType,
};

const ARRAY_UNIFORM_SIZE: usize = 16;
const NUM_BUCKETS: usize = ARRAY_UNIFORM_SIZE * 4;

#[derive(Resource)]
pub struct CfgResource(MyConfig);

fn main() {
    let config = confy::load("bevy_audioviz", "config").unwrap();
    println!(
        "Config file location: {:#?}",
        confy::get_configuration_file_path("bevy_audioviz", "config").unwrap()
    );
    println!("{:?}", config);
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Audio Visualization".into(),
                //resolution: (500., 300.).into(),
                present_mode: PresentMode::AutoVsync,
                // Tells wasm to resize the window according to the available canvas
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }),))
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(AudioVisualizerState::new(NUM_BUCKETS))
        .insert_resource(CfgResource(config))
        .init_resource::<AudioReceiver>() // Initialize the `AudioReceiver` resource.
        .init_resource::<VisualizationType>()
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_visualization)
        .add_systems(Update, visualization_toggle_system)
        .add_systems(Update, window_resized_event.after(spawn_visualization))
        .add_systems(Update, audio_capture_startup_system)
        .add_systems(
            Update,
            audio_event_system
                .after(audio_capture_startup_system)
                .before(visualization_toggle_system)
                .before(spawn_visualization),
        )
        .add_systems(Update, toggle_vsync)
        .init_resource::<AudioEntity>()
        .init_resource::<CircleEntity>()
        .init_resource::<CircleSplitEntity>()
        .init_resource::<PolygonEntity>()
        .add_plugins(Material2dPlugin::<AudioMaterial>::default())
        .add_plugins(Material2dPlugin::<CircleMaterial>::default())
        .add_plugins(Material2dPlugin::<CircleSplitMaterial>::default())
        .add_plugins(Material2dPlugin::<PolygonMaterial>::default())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
/// This system toggles the vsync mode when pressing the button V.
/// You'll see fps increase displayed in the console.
fn toggle_vsync(input: Res<Input<KeyCode>>, mut windows: Query<&mut Window>) {
    if input.just_pressed(KeyCode::V) {
        let mut window = windows.single_mut();

        window.present_mode = if matches!(window.present_mode, PresentMode::AutoVsync) {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::AutoVsync
        };
        info!("PRESENT_MODE: {:?}", window.present_mode);
    }
}
