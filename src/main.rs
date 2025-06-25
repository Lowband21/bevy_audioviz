#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
// Removed unused imports
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::pbr::MaterialPlugin;
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::window::{PresentMode, WindowTheme};
use log::{debug, error, info, warn};

use bevy_egui::EguiPlugin;

mod audio_capture;
mod audio_processing;
mod cfg;
mod materials;
mod ui;
mod visualization;

use crate::audio_capture::{audio_capture_startup_system, AudioReceiver};
use crate::audio_processing::{audio_event_system, AudioVisualizerState};
use crate::cfg::*;
use crate::materials::{BarEntity, BarMaterial};
use crate::materials::{CircleSplitEntity, CircleSplitMaterial};
use crate::materials::{PolygonEntity, PolygonMaterial};
use crate::materials::{StringEntity, StringMaterial};
use crate::materials::{WaveEntity, WaveMaterial};
use crate::ui::{Colors, UIPlugin};
use crate::visualization::{
    spawn_visualization, visualization_toggle_system, window_resized_event, VisualizationType,
};
use cpal::available_hosts;
use cpal::traits::{DeviceTrait, HostTrait};

const ARRAY_UNIFORM_SIZE: usize = 16;
const NUM_BUCKETS: usize = ARRAY_UNIFORM_SIZE * 4;

#[derive(Resource)]
pub struct CfgResource(MyConfig);

#[derive(Resource, Default)]
pub struct GUIToggle {
    pub active: bool,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AudioVizSystem {
    Audio,
    Visualization,
    Input,
}

fn main() {
    let config = match confy::load("bevy_audioviz", "config") {
        Ok(config) => config,
        Err(_) => MyConfig::default(),
    };
    info!(
        "Config file location: {:#?}",
        confy::get_configuration_file_path("bevy_audioviz", "config").unwrap()
    );
    debug!("Loaded config: {:?}", config);
    list_available_hosts();
    list_audio_devices();

    let mut default_plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Audio Visualization".into(),
            present_mode: PresentMode::AutoVsync,
            prevent_default_event_handling: false,
            window_theme: Some(WindowTheme::Dark),
            ..default()
        }),
        ..default()
    });

    // Disable asset watcher for release builds
    if !cfg!(debug_assertions) {
        default_plugins = default_plugins.set(AssetPlugin {
            watch_for_changes_override: Some(false), // Explicitly disable watching
            ..default()
        });
    }

    App::new()
        .add_plugins(default_plugins) // Add the potentially modified group
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(UIPlugin)
        .insert_resource(AudioVisualizerState::new(NUM_BUCKETS))
        .insert_resource(CfgResource(config))
        .insert_resource(GUIToggle::default())
        .insert_resource(Colors::default())
        .init_resource::<AudioReceiver>()
        .init_resource::<VisualizationType>()
        .add_systems(Startup, setup)
        .configure_sets(
            Update,
            (
                AudioVizSystem::Audio,
                AudioVizSystem::Visualization,
                AudioVizSystem::Input,
            ),
        )
        .add_systems(
            Update,
            (audio_capture_startup_system, audio_event_system).in_set(AudioVizSystem::Audio),
        )
        .add_systems(
            Update,
            (
                spawn_visualization,
                visualization_toggle_system,
                window_resized_event,
            )
                .in_set(AudioVizSystem::Visualization),
        )
        .add_systems(
            Update,
            (toggle_vsync, toggle_gui).in_set(AudioVizSystem::Input),
        )
        .init_resource::<BarEntity>()
        .init_resource::<StringEntity>()
        .init_resource::<CircleSplitEntity>()
        .init_resource::<PolygonEntity>()
        .init_resource::<WaveEntity>()
        .add_plugins(MaterialPlugin::<BarMaterial>::default())
        .add_plugins(MaterialPlugin::<StringMaterial>::default())
        .add_plugins(MaterialPlugin::<CircleSplitMaterial>::default())
        .add_plugins(MaterialPlugin::<PolygonMaterial>::default())
        .add_plugins(MaterialPlugin::<WaveMaterial>::default())
        .run();
}

fn setup(mut commands: Commands) {
    // Position 3D camera to replicate 2D view with orthographic projection
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            near: -10.0,  // Reduced near/far range for better depth precision
            far: 10.0,
            viewport_origin: Vec2::new(0.5, 0.5),
            scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
            area: Rect::new(-1.0, -1.0, 1.0, 1.0),
        }),
    ));
}

fn toggle_vsync(keyboard: Res<ButtonInput<KeyCode>>, mut windows: Query<&mut Window>) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        if let Ok(mut window) = windows.single_mut() {
            window.present_mode = if matches!(window.present_mode, PresentMode::AutoVsync) {
                PresentMode::AutoNoVsync
            } else {
                PresentMode::AutoVsync
            };
            info!("PRESENT_MODE: {:?}", window.present_mode);
        }
    }
}

fn list_available_hosts() {
    info!("Available hosts:");
    for host_id in available_hosts() {
        info!("{:?}", host_id);
    }
}

fn list_audio_devices() {
    let host = cpal::default_host();
    let input_devices = host.input_devices().unwrap();
    let output_devices = host.output_devices().unwrap();

    info!("Input Devices:");
    for device in input_devices {
        info!("{}", device.name().unwrap());
    }

    info!("Output Devices:");
    for device in output_devices {
        info!("{}", device.name().unwrap());
    }
}

fn toggle_gui(keyboard: Res<ButtonInput<KeyCode>>, mut toggle: ResMut<GUIToggle>) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        toggle.active = !toggle.active;
        info!("GUI Toggled: {}", toggle.active);
    }
}
