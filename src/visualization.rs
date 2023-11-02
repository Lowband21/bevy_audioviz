use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::{PrimaryWindow, WindowResized};

use crate::audio_capture::AudioThreadFlag;
use crate::audio_capture::{stream_input, DeviceType};
use crate::bar_material::*;
use crate::circle_material::*;
use crate::AudioReceiver;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(Resource)]
pub enum VisualizationType {
    Bar,
    Circle,
}

impl Default for VisualizationType {
    fn default() -> Self {
        VisualizationType::Bar
    }
}

// visualization_toggle_system now ensures proper cleanup and restart
pub fn visualization_toggle_system(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut visualization_type: ResMut<VisualizationType>,
    audio_receiver_res: Option<ResMut<AudioReceiver>>,
    audio_thread_flag: Option<Res<AudioThreadFlag>>,
) {
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
            VisualizationType::Bar => VisualizationType::Circle,
            VisualizationType::Circle => VisualizationType::Bar,
        };

        // Restart the audio thread with a new run flag
        let new_run_flag = Arc::new(AtomicBool::new(true));
        let (audio_receiver, thread_handle) =
            stream_input(DeviceType::Output, 2048, new_run_flag.clone());
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
    mut meshes: ResMut<Assets<Mesh>>, // For meshes
    mut audio_material: ResMut<Assets<AudioMaterial>>,
    mut circle_material: ResMut<Assets<CircleMaterial>>,
    mut audio_entity: ResMut<AudioEntity>,
    mut circle_entity: ResMut<CircleEntity>,
    visualization_type: Res<VisualizationType>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    if visualization_type.is_changed() {
        println!("Fractal Type Changed");

        // Variable setup
        let window = primary_window.single();
        let window_size = Vec2::new(window.width(), window.height());
        let mesh = Mesh::from(shape::Quad {
            size: window_size,
            flip: false,
        });
        let audio_mesh: Mesh2dHandle = Mesh2dHandle(meshes.add(mesh.clone()));

        match *visualization_type {
            VisualizationType::Bar => {
                // Remove the old visualizer entity if it exists
                if let Some(entity) = audio_entity.0.take() {
                    commands.entity(entity).despawn();
                }
                if let Some(entity) = circle_entity.0.take() {
                    commands.entity(entity).despawn();
                }
                // Spawn Mandelbrot entity
                let bar_material_handle =
                    prepare_audio_material(&mut audio_material, window_size.x, window_size.y);
                audio_entity.0 = Some(
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
            VisualizationType::Circle => {
                // Remove the old visualizer entity if it exists
                if let Some(entity) = audio_entity.0.take() {
                    commands.entity(entity).despawn();
                }
                if let Some(entity) = circle_entity.0.take() {
                    commands.entity(entity).despawn();
                }
                // Spawn Mandelbrot entity
                let circle_material_handle =
                    prepare_circle_material(&mut circle_material, window_size.x, window_size.y);
                circle_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: audio_mesh.clone(),
                            material: circle_material_handle,
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
    mut meshes: ResMut<Assets<Mesh>>, // For meshes
    visualization_type: Res<VisualizationType>,
    mut audio_material: ResMut<Assets<AudioMaterial>>,
    mut circle_material: ResMut<Assets<CircleMaterial>>,
    mut audio_entity: ResMut<AudioEntity>,
    mut circle_entity: ResMut<CircleEntity>,
) {
    for event in events.iter() {
        println!("Updating Window Size");

        // Despawn any existing visualizer entities regardless of type.
        if let Some(entity) = audio_entity.0.take() {
            commands.entity(entity).despawn();
        }
        if let Some(entity) = circle_entity.0.take() {
            commands.entity(entity).despawn();
        }

        // Create a new mesh for the updated window size.
        let mesh_handle = meshes.add(Mesh::from(shape::Quad {
            size: Vec2::new(event.width, event.height),
            flip: false,
        }));

        // Spawn entities based on the current visualization type.
        match *visualization_type {
            VisualizationType::Bar => {
                // Prepare and spawn a new bar visualizer entity.
                let bar_material_handle =
                    prepare_audio_material(&mut audio_material, event.width, event.height);
                audio_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: Mesh2dHandle(mesh_handle),
                            material: bar_material_handle,
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            ..Default::default()
                        })
                        .id(),
                );
            }
            VisualizationType::Circle => {
                // Prepare and spawn a new circle visualizer entity.
                let circle_material_handle =
                    prepare_circle_material(&mut circle_material, event.width, event.height);
                circle_entity.0 = Some(
                    commands
                        .spawn(MaterialMesh2dBundle {
                            mesh: Mesh2dHandle(mesh_handle),
                            material: circle_material_handle,
                            transform: Transform::from_xyz(0.0, 0.0, 0.0),
                            ..Default::default()
                        })
                        .id(),
                );
            }
        }
    }
}
