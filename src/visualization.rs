use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::{PrimaryWindow, WindowResized};
use bevy::math::primitives::Rectangle;

use crate::audio_capture::AudioThreadFlag;
use crate::audio_capture::{stream_input, DeviceType};
use crate::materials::*;
use crate::AudioReceiver;
use crate::CfgResource;
use crate::Colors;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::{impl_material_new, impl_one_channel_material_new, prepare_material};

// Move material implementations to module level
impl_material_new!(StringMaterial);
impl_material_new!(CircleSplitMaterial);
impl_material_new!(WaveMaterial);
impl_one_channel_material_new!(BarMaterial);
impl_one_channel_material_new!(PolygonMaterial);

#[derive(Resource)]
#[derive(Default)]
pub enum VisualizationType {
    #[default]
    Bar,
    String,
    CircleSplit,
    Polygon,
    Wave,
}

// visualization_toggle_system now ensures proper cleanup and restart
pub fn visualization_toggle_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut visualization_type: ResMut<VisualizationType>,
    audio_receiver_res: Option<ResMut<AudioReceiver>>,
    audio_thread_flag: Option<Res<AudioThreadFlag>>,
    config: Res<CfgResource>,
) {
    let config = &config.0;
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Signal the audio thread to stop before changing the visualization type
        if let Some(mut audio_receiver) = audio_receiver_res {
            if let Some(flag) = audio_thread_flag {
                // Signal the audio thread to stop
                flag.0.store(false, Ordering::SeqCst);
            }

            if let Some(thread_handle) = audio_receiver.thread_handle.take() {
                // Safely take and join the audio thread
                thread_handle.join().expect("Failed to join audio thread");
            }
        }

        commands.remove_resource::<AudioThreadFlag>();
        commands.remove_resource::<AudioReceiver>();

        *visualization_type = match *visualization_type {
            VisualizationType::Bar => VisualizationType::String,
            VisualizationType::String => VisualizationType::CircleSplit,
            VisualizationType::CircleSplit => VisualizationType::Wave,
            VisualizationType::Wave => VisualizationType::Polygon,
            VisualizationType::Polygon => VisualizationType::Bar,
        };

        // Restart the audio thread with a new run flag
        let new_run_flag = Arc::new(AtomicBool::new(true));
        let (audio_receiver, thread_handle) =
            stream_input(DeviceType::Output, new_run_flag.clone(), config);
        commands.insert_resource(AudioThreadFlag(new_run_flag));
        commands.insert_resource(AudioReceiver {
            receiver: Arc::new(Mutex::new(audio_receiver)),
            thread_handle: Some(thread_handle), // Store the new thread handle
        });

        // It's better to handle audio data in a separate system or thread to avoid blocking
    }
}

pub fn spawn_visualization(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut bar_material: ResMut<Assets<BarMaterial>>,
    mut bar_entity: ResMut<BarEntity>,
    mut string_material: ResMut<Assets<StringMaterial>>,
    mut string_entity: ResMut<StringEntity>,
    mut circle_split_material: ResMut<Assets<CircleSplitMaterial>>,
    mut circle_split_entity: ResMut<CircleSplitEntity>,
    mut polygon_material: ResMut<Assets<PolygonMaterial>>,
    mut polygon_entity: ResMut<PolygonEntity>,
    mut wave_material: ResMut<Assets<WaveMaterial>>,
    mut wave_entity: ResMut<WaveEntity>,
    visualization_type: Res<VisualizationType>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    colors: Res<Colors>,
) {
    if visualization_type.is_changed() {
        println!("Fractal Type Changed");

        // Variable setup
        let colors = colors.into_inner();
        let window = primary_window.single();
        let window_size = Vec2::new(window.width(), window.height());
        let polygon_window_size = Vec2::new(window.height(), window.width());
        let mesh = Mesh::from(Rectangle::new(window_size.x, window_size.y));
        let polygon_mesh = Mesh::from(Rectangle::new(polygon_window_size.x, polygon_window_size.y));
        let audio_mesh: Mesh2dHandle = Mesh2dHandle(meshes.add(mesh.clone()));
        let polygon_audio_mesh: Mesh2dHandle = Mesh2dHandle(meshes.add(polygon_mesh.clone()));

        // Remove the old visualizer entity if it exists
        if let Some(entity) = bar_entity.0.take() {
            commands.entity(entity).despawn();
        }
        if let Some(entity) = string_entity.0.take() {
            commands.entity(entity).despawn();
        }
        if let Some(entity) = circle_split_entity.0.take() {
            commands.entity(entity).despawn();
        }
        if let Some(entity) = polygon_entity.0.take() {
            commands.entity(entity).despawn();
        }
        if let Some(entity) = wave_entity.0.take() {
            commands.entity(entity).despawn();
        }

        match *visualization_type {
            VisualizationType::Bar => {
                let bar_material_handle = prepare_material!(
                    BarMaterial,
                    bar_material,
                    window_size.x,
                    window_size.y,
                    colors
                );
                bar_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: audio_mesh.clone(),
                            material: bar_material_handle,
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            ..Default::default()
                        })
                        .id(),
                );
            }
            VisualizationType::String => {
                let string_material_handle = prepare_material!(
                    StringMaterial,
                    &mut string_material,
                    window_size.x,
                    window_size.y,
                    colors
                );
                string_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: audio_mesh.clone(),
                            material: string_material_handle,
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            ..Default::default()
                        })
                        .id(),
                );
            }
            VisualizationType::CircleSplit => {
                let circle_split_material_handle = prepare_material!(
                    CircleSplitMaterial,
                    &mut circle_split_material,
                    window_size.x,
                    window_size.y,
                    colors
                );
                circle_split_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: audio_mesh.clone(),
                            material: circle_split_material_handle,
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            ..Default::default()
                        })
                        .id(),
                );
            }
            VisualizationType::Polygon => {
                let polygon_material_handle = prepare_material!(
                    PolygonMaterial,
                    &mut polygon_material,
                    polygon_window_size.x,
                    polygon_window_size.y,
                    colors
                );
                polygon_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: polygon_audio_mesh.clone(),
                            material: polygon_material_handle,
                            transform: Transform::from_rotation(Quat::from_rotation_z(
                                (90.0_f32).to_radians(),
                            )),
                            ..Default::default()
                        })
                        .id(),
                );
            }
            VisualizationType::Wave => {
                let wave_material_handle = prepare_material!(
                    WaveMaterial,
                    wave_material,
                    window_size.x,
                    window_size.y,
                    colors
                );
                wave_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: audio_mesh.clone(),
                            material: wave_material_handle,
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            ..Default::default()
                        })
                        .id(),
                );
            }
        }

        println!("Spawned Audio Visualization");
    }
}

pub fn window_resized_event(
    mut events: EventReader<WindowResized>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    visualization_type: Res<VisualizationType>,
    mut bar_material: ResMut<Assets<BarMaterial>>,
    mut bar_entity: ResMut<BarEntity>,
    mut string_material: ResMut<Assets<StringMaterial>>,
    mut string_entity: ResMut<StringEntity>,
    mut circle_split_material: ResMut<Assets<CircleSplitMaterial>>,
    mut circle_split_entity: ResMut<CircleSplitEntity>,
    mut polygon_material: ResMut<Assets<PolygonMaterial>>,
    mut polygon_entity: ResMut<PolygonEntity>,
    mut wave_material: ResMut<Assets<WaveMaterial>>,
    mut wave_entity: ResMut<WaveEntity>,
    colors: Res<Colors>,
) {
    let colors = colors.into_inner();
    for event in events.read() {
        println!("Updating Window Size");

        // Create meshes for the new window size
        let mesh_handle = meshes.add(Mesh::from(Rectangle::new(event.width, event.height)));
        let polygon_mesh_handle = meshes.add(Mesh::from(Rectangle::new(event.height, event.width)));

        // First, despawn all existing entities if they exist
        if let Some(entity) = bar_entity.0.take() {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn();
            }
        }
        if let Some(entity) = string_entity.0.take() {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn();
            }
        }
        if let Some(entity) = circle_split_entity.0.take() {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn();
            }
        }
        if let Some(entity) = polygon_entity.0.take() {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn();
            }
        }
        if let Some(entity) = wave_entity.0.take() {
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).despawn();
            }
        }

        // Then spawn the new entity based on the current visualization type
        match *visualization_type {
            VisualizationType::Bar => {
                let bar_material_handle = prepare_material!(
                    BarMaterial,
                    bar_material,
                    event.width,
                    event.height,
                    colors
                );
                let new_entity = commands
                    .spawn(MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(mesh_handle),
                        material: bar_material_handle,
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        ..Default::default()
                    })
                    .id();
                bar_entity.0 = Some(new_entity);
            }
            VisualizationType::String => {
                let string_material_handle = prepare_material!(
                    StringMaterial,
                    string_material,
                    event.width,
                    event.height,
                    colors
                );
                let new_entity = commands
                    .spawn(MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(mesh_handle),
                        material: string_material_handle,
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        ..Default::default()
                    })
                    .id();
                string_entity.0 = Some(new_entity);
            }
            VisualizationType::CircleSplit => {
                let circle_split_material_handle = prepare_material!(
                    CircleSplitMaterial,
                    circle_split_material,
                    event.width,
                    event.height,
                    colors
                );
                let new_entity = commands
                    .spawn(MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(mesh_handle),
                        material: circle_split_material_handle,
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        ..Default::default()
                    })
                    .id();
                circle_split_entity.0 = Some(new_entity);
            }
            VisualizationType::Polygon => {
                let polygon_material_handle = prepare_material!(
                    PolygonMaterial,
                    polygon_material,
                    event.width,
                    event.height,
                    colors
                );
                let new_entity = commands
                    .spawn(MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(polygon_mesh_handle),
                        material: polygon_material_handle,
                        transform: Transform::from_rotation(Quat::from_rotation_z(
                            (90.0_f32).to_radians(),
                        )),
                        ..Default::default()
                    })
                    .id();
                polygon_entity.0 = Some(new_entity);
            }
            VisualizationType::Wave => {
                let wave_material_handle = prepare_material!(
                    WaveMaterial,
                    wave_material,
                    event.width,
                    event.height,
                    colors
                );
                let new_entity = commands
                    .spawn(MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(mesh_handle),
                        material: wave_material_handle,
                        transform: Transform::from_xyz(0.0, 0.0, 0.0),
                        ..Default::default()
                    })
                    .id();
                wave_entity.0 = Some(new_entity);
            }
        }
    }
}
