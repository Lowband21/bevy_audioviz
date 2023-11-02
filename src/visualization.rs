use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::{PrimaryWindow, WindowResized};

use crate::bar_material::{prepare_audio_material, AudioEntity, AudioMaterial};

pub fn spawn_visualization(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, // For meshes
    mut audio_material: ResMut<Assets<AudioMaterial>>,
    mut audio_entity: ResMut<AudioEntity>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = primary_window.single();
    let window_size = Vec2::new(window.width(), window.height());
    let mesh = Mesh::from(shape::Quad {
        size: window_size,
        flip: false,
    });
    let audio_mesh: Mesh2dHandle = Mesh2dHandle(meshes.add(mesh.clone()));

    // Spawn Mandelbrot entity
    let mandelbrot_material_handle =
        prepare_audio_material(&mut audio_material, window_size.x, window_size.y);
    audio_entity.0 = Some(
        commands
            .spawn(MaterialMesh2dBundle {
                mesh: audio_mesh.clone(),
                material: mandelbrot_material_handle,
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            })
            .id(),
    );
    println!("Spawned Audio Visualization");
}

pub fn window_resized_event(
    mut events: EventReader<WindowResized>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, // For meshes
    mut audio_material: ResMut<Assets<AudioMaterial>>,
    mut audio_entity: ResMut<AudioEntity>,
) {
    for event in events.iter() {
        println!("Updating Window Size");
        // Remove the old visualizer entity if it exists
        if let Some(entity) = audio_entity.0 {
            commands.entity(entity).despawn();
        }
        let mesh = Mesh::from(shape::Quad {
            size: Vec2::new(event.width, event.height),
            flip: false,
        });
        let audio_mesh: Mesh2dHandle = Mesh2dHandle(meshes.add(mesh.clone()));

        // Spawn Mandelbrot entity
        let mandelbrot_material_handle =
            prepare_audio_material(&mut audio_material, event.width, event.height);
        audio_entity.0 = Some(
            commands
                .spawn(MaterialMesh2dBundle {
                    mesh: audio_mesh.clone(),
                    material: mandelbrot_material_handle,
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..Default::default()
                })
                .id(),
        );
    }
}
