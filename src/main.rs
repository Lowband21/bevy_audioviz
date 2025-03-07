#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;
use bevy::window::{PresentMode, WindowTheme};
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;

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
    println!(
        "Config file location: {:#?}",
        confy::get_configuration_file_path("bevy_audioviz", "config").unwrap()
    );
    println!("{:?}", config);
    list_available_hosts();
    list_audio_devices();

    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Audio Visualization".into(),
                present_mode: PresentMode::AutoVsync,
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                ..default()
            }),
            ..default()
        }).set(AssetPlugin {
            watch_for_changes_override: None,
            ..default()
        }),))
        .add_plugins(EguiPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(UIPlugin)
        .insert_resource(AudioVisualizerState::new(NUM_BUCKETS))
        .insert_resource(CfgResource(config))
        .insert_resource(GUIToggle::default())
        .insert_resource(Colors::default())
        .init_resource::<AudioReceiver>()
        .init_resource::<VisualizationType>()
        .add_systems(Startup, setup)
        .configure_sets(Update, (
            AudioVizSystem::Audio,
            AudioVizSystem::Visualization,
            AudioVizSystem::Input,
        ))
        .add_systems(Update, (
            audio_capture_startup_system,
            audio_event_system
        ).in_set(AudioVizSystem::Audio))
        .add_systems(Update, (
            spawn_visualization,
            visualization_toggle_system,
            window_resized_event
        ).in_set(AudioVizSystem::Visualization))
        .add_systems(Update, (
            toggle_vsync,
            toggle_gui
        ).in_set(AudioVizSystem::Input))
        .init_resource::<BarEntity>()
        .init_resource::<StringEntity>()
        .init_resource::<CircleSplitEntity>()
        .init_resource::<PolygonEntity>()
        .init_resource::<WaveEntity>()
        .add_plugins(Material2dPlugin::<BarMaterial>::default())
        .add_plugins(Material2dPlugin::<StringMaterial>::default())
        .add_plugins(Material2dPlugin::<CircleSplitMaterial>::default())
        .add_plugins(Material2dPlugin::<PolygonMaterial>::default())
        .add_plugins(Material2dPlugin::<WaveMaterial>::default())
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn toggle_vsync(keyboard: Res<ButtonInput<KeyCode>>, mut windows: Query<&mut Window>) {
    if keyboard.just_pressed(KeyCode::KeyV) {
        let mut window = windows.single_mut();

        window.present_mode = if matches!(window.present_mode, PresentMode::AutoVsync) {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::AutoVsync
        };
        info!("PRESENT_MODE: {:?}", window.present_mode);
    }
}

fn list_available_hosts() {
    println!("Available hosts:");
    for host_id in available_hosts() {
        println!("{:?}", host_id);
    }
}

fn list_audio_devices() {
    let host = cpal::default_host();
    let input_devices = host.input_devices().unwrap();
    let output_devices = host.output_devices().unwrap();

    println!("Input Devices:");
    for device in input_devices {
        println!("{}", device.name().unwrap());
    }

    println!("\nOutput Devices:");
    for device in output_devices {
        println!("{}", device.name().unwrap());
    }
}

fn toggle_gui(keyboard: Res<ButtonInput<KeyCode>>, mut toggle: ResMut<GUIToggle>) {
    if keyboard.just_pressed(KeyCode::KeyG) {
        toggle.active = !toggle.active;
        info!("GUI Toggled: {}", toggle.active);
    }
}
