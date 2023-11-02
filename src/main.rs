use std::sync::mpsc::channel;
use bevy::prelude::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2dPlugin;
use bevy::sprite::{Material2d};
use bevy::sprite::Mesh2dHandle;
use bevy::sprite::MaterialMesh2dBundle;

use rustfft::{num_complex::Complex, FftPlanner};
use std::thread;
use std::sync::mpsc::Receiver;
use bevy::window::{PrimaryWindow, WindowResized};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(AudioResource {
            data: Arc::new(Mutex::new(Vec::new())),
        })
        .insert_resource(AudioVisualizerState::new(32))
        .init_resource::<AudioEntity>()
        .init_resource::<AudioReceiver>() // Initialize the `AudioReceiver` resource.
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_visualization)
        .add_systems(Update, window_resized_event)
        //.add_systems(Startup, setup_audio_stream)

        .add_systems(Startup, audio_capture_startup_system)
        .add_systems(Update, audio_event_system)

        //.add_systems(Update, update_uniforms)
        .add_plugins(Material2dPlugin::<AudioMaterial>::default())
        .run();
}

#[derive(Resource)]
pub struct AudioEntity(pub Option<Entity>);
impl Default for AudioEntity{
    fn default() -> Self {
        AudioEntity(None)
    }
}

pub struct AudioUniforms {
    normalized_data: Vec<f32>,
}

#[derive(Component, Debug, Clone, AsBindGroup, TypeUuid, TypePath)]
#[uuid = "e71681d9-3499-4bba-881d-2eaeed7c1c31"]
pub struct AudioMaterial {
    #[uniform(0)]
    normalized_data: [Vec4; 8], // Use an array of vec4s (which is an array of [f32; 4] in Rust)}
    #[uniform(1)]
    viewport_width: f32,
    #[uniform(2)]
    viewport_height: f32,
}
impl Material2d for AudioMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/audio_fragment.wgsl".into()
    }
}
#[derive(Resource)]
struct AudioResource {
    data: Arc<Mutex<Vec<f32>>>,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
fn update_uniforms(audio_resource: Res<AudioResource>, mut materials: ResMut<Assets<AudioMaterial>>) {
    let data_arc = Arc::clone(&audio_resource.data);
    let audio_data = data_arc.lock().unwrap();
    println!("Audio data: {:#?}", audio_data);
    let max_value = audio_data.iter().cloned().fold(f32::MIN, f32::max);
    let normalized_data: Vec<f32> = audio_data.iter().map(|&x| x / max_value).collect();

    // Assuming we have single-channel audio data, we'll distribute it across the vec4 components
    // Calculate the number of samples per component
    let samples_per_component = normalized_data.len() / 32; // 32 components in total (8 vec4s)
    let mut buckets = [Vec4::ZERO; 8]; // An array of 8 Vec4s initialized to zero

    for bucket_index in 0..8 {
        let mut bucket_data = [0.0; 4]; // Temporary array to hold data for the Vec4
        for component_index in 0..4 {
            let start = (bucket_index * 4 + component_index) * samples_per_component;
            let end = usize::min(start + samples_per_component, normalized_data.len());
            let component_slice = &normalized_data[start..end];
            bucket_data[component_index] = component_slice.iter().sum::<f32>() / component_slice.len() as f32;
        }
        // Store the computed averages in the Vec4
        buckets[bucket_index] = Vec4::from(bucket_data);
    }

    // Applying Decay for each Vec4
    let decay_factor = 0.9;
    for bucket_index in 1..8 {
        buckets[bucket_index] = buckets[bucket_index - 1] * decay_factor + buckets[bucket_index] * (1.0 - decay_factor);
    }

    for (_, material) in materials.iter_mut() {
        material.normalized_data = buckets.clone();
    }
    //println!("Updated buckets to: {:#?}", buckets);
}


use bevy::ecs::entity::Entities;


fn spawn_visualization(
    entities: &Entities,
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
    let mandelbrot_material_handle = prepare_audio_material(
        &mut audio_material, window_size.x, window_size.y
    );
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

fn window_resized_event(mut events: EventReader<WindowResized>,
                        entities: &Entities,
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
        let mandelbrot_material_handle = prepare_audio_material(
            &mut audio_material, event.width, event.height,
        );
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
// Utility function to prepare and return a Mandelbrot material with the given uniforms.
pub fn prepare_audio_material(
    materials: &mut ResMut<Assets<AudioMaterial>>,
    width: f32,
    height: f32,
) -> Handle<AudioMaterial> {
    let material = AudioMaterial {
        normalized_data: [Vec4::new(0.0, 0.0, 0.0, 0.0); 8],
        viewport_width: width,
        viewport_height: height,
    };
    materials.add(material)
}
// Define a new Bevy event to communicate audio data to the main thread.
#[derive(Event)]
pub struct AudioProcessedEvent(Vec<Vec<f32>>);
#[derive(Debug)]
pub enum DeviceType {
    Input(),
    Output(),
}
// Adapted `stream_input` to use Bevy's event system.
// This function initializes the audio input stream and returns a receiver for audio events.
pub fn stream_input(
    device_type: DeviceType,
    buffer_size: usize, // Add this parameter to specify the buffer size
) -> Receiver<AudioProcessedEvent> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        let host = cpal::default_host();
        let device = match device_type {
            DeviceType::Input() => host.default_input_device().expect("No default input device"),
            DeviceType::Output() => host.default_output_device().expect("No default output device"),
        };

        let config = device.default_input_config().expect("Failed to get default input config");

        let err_fn = |err| eprintln!("An error occurred on the audio stream: {}", err);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(&config.into(), move |data: &[f32], _: &_| {
                // Create a buffer with the specified buffer size
                let mut buffer = vec![0.0; buffer_size];

                // Fill the buffer with the incoming audio data
                for (i, &sample) in data.iter().enumerate().take(buffer_size) {
                    buffer[i] = sample;
                }

                // Send the buffer to the main Bevy thread through the channel
                // Here the buffer is split into chunks of size that can be changed as needed
                sender.send(AudioProcessedEvent(buffer.chunks_exact(4).map(|c| c.to_vec()).collect()))
                    .expect("Failed to send audio data");
            }, err_fn, None).expect("Failed to build audio input stream"),
            _ => panic!("Unsupported sample format"),
        };

        stream.play().expect("Failed to play audio stream");
        thread::park();
    });

    receiver
}



// Define the error handling function for the audio stream.
fn err_fn(err: cpal::StreamError) {
    eprintln!("An error occurred on the audio stream: {}", err);
}

// Define a wrapper around `mpsc::Receiver` to make it a Bevy resource.
#[derive(Resource)]
pub struct AudioReceiver {
    pub receiver: Arc<Mutex<Receiver<AudioProcessedEvent>>>,
}

// Implement `FromWorld` for the new resource type.
impl FromWorld for AudioReceiver {
    fn from_world(_: &mut World) -> Self {
        // Here you would call your `stream_input` function to create the receiver.
        let (sender, receiver) = channel();

        // Set up the audio streaming in a separate thread.
        // ...

        AudioReceiver {
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}
// Bevy system to initialize audio capture and store the receiver in a resource.
fn audio_capture_startup_system(
    mut commands: Commands,
) {
    // Retrieve the receiver from the `stream_input` function.
    let audio_receiver = AudioReceiver{receiver: Arc::new(Mutex::new(stream_input(DeviceType::Output(), 2048)))};

    // Insert the receiver into Bevy's resource system for later access.
    commands.insert_resource(audio_receiver);
}
#[derive(Resource)]
struct AudioVisualizerState {
    previous_buckets: Vec<f32>,
}

impl AudioVisualizerState {
    fn new(num_buckets: usize) -> Self {
        AudioVisualizerState {
            previous_buckets: vec![0.0; num_buckets],
        }
    }

    fn animate_buckets(&mut self, current_buckets: &[f32], interpolation_factor: f32) -> Vec<f32> {
        let mut animated_buckets = Vec::with_capacity(current_buckets.len());

        for (&current, previous) in current_buckets.iter().zip(self.previous_buckets.iter_mut()) {
            // Interpolate between the previous bucket value and the current one
            let interpolated_value = *previous + (current - *previous) * interpolation_factor;
            animated_buckets.push(interpolated_value);
            // Update the previous value for the next frame
            *previous = interpolated_value;
        }

        animated_buckets
    }
}


// Entry function for the audio event system
fn audio_event_system(
    audio_receiver: Res<AudioReceiver>,
    mut materials: ResMut<Assets<AudioMaterial>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut visualizer_state: ResMut<AudioVisualizerState>,
) {
    if let Some(window) = primary_window.iter().next() {
        let window_size = Vec2::new(window.width(), window.height());

        if window_size.x > 0.0 && window_size.y > 0.0 {
            if let Ok(audio_event) = audio_receiver.receiver.lock().unwrap().try_recv() {
                let mut fft_planner = FftPlanner::new();
                let fft = fft_planner.plan_fft_forward(2048);

                // Convert audio samples to complex numbers for FFT
                // Assuming audio_event.0 is a Vec<Vec4>, where Vec4 is a type with four f32 fields
                let mut input: Vec<Complex<f32>> = audio_event.0
                    .iter()
                    // flatten Vec<Vec4> into an iterator of f32
                    .flat_map(|vec| vec.iter())
                    // map each f32 to a Complex<f32>
                    .map(|&sample| Complex::new(sample, 0.0))
                    .collect();
                // Apply a window function to the audio samples before FFT
                apply_hann_window(&mut input);

                // Ensure that the input buffer isn't empty and has a length that's a power of two
                if !input.is_empty() && input.len().is_power_of_two() {
                    // Perform FFT
                    fft.process(&mut input);

                    // Convert FFT output to magnitude and bucket into 32 ranges
                    let mut buckets = bucketize_fft_to_ranges(&input, 32, 44000);

                    // Apply smoothing to the buckets
                    let smoothing = 2;
                    let smoothing_size = 4;
                    smooth(&mut buckets, smoothing, smoothing_size);

                    // Animate bucket transitions
                    let interpolation_factor = 0.5; // Adjust this value as needed
                    let animated_buckets = visualizer_state.animate_buckets(&buckets, interpolation_factor);

                    // Normalize animated buckets for visualization
                    let normalized_buckets = normalize_buckets(&animated_buckets);

                    // Update the material properties
                    for (_, material) in materials.iter_mut() {
                        material.normalized_data = normalized_buckets;
                        material.viewport_width = window_size.x;
                        material.viewport_height = window_size.y;
                    }
                }
            }
        }
    }
}
fn apply_hann_window(input: &mut Vec<Complex<f32>>) {
    let len = input.len();
    for (i, sample) in input.iter_mut().enumerate() {
        let window_value = 0.5 * (1.0 - Float::cos(2.0 * std::f32::consts::PI * i as f32 / (len - 1) as f32));
        *sample *= Complex::new(window_value, 0.0);
    }
}
use rustfft::num_traits::Float; // Import the Float trait
fn bucketize_fft_to_ranges(input: &[Complex<f32>], num_buckets: usize, sample_rate: usize) -> Vec<f32> {
    let mut buckets = vec![0f32; num_buckets];
    let half_len = input.len() / 2;

    let min_log_freq = 20f32.log2(); // Log2 of 20 Hz
    let max_log_freq = (sample_rate as f32 / 2.0).log2(); // Log2 of Nyquist frequency

    // Iterate over the first half of the FFT output
    for (i, bin) in input.iter().enumerate().take(half_len) {
        let freq = i as f32 * sample_rate as f32 / input.len() as f32; // Frequency of the FFT bin
        let log_freq = freq.log2();

        // Calculate the bucket index based on the logarithmic frequency
        let bucket_index = ((log_freq - min_log_freq) / (max_log_freq - min_log_freq) * num_buckets as f32).floor() as usize;

        if bucket_index < buckets.len() {
            buckets[bucket_index] += bin.norm_sqr(); // Add squared magnitude to the bucket
        }
    }

    // Compute the average magnitude for each bucket
    for value in &mut buckets {
        *value = value.sqrt();
    }

    buckets
}



fn normalize_buckets(buckets: &[f32]) -> [Vec4; 8] {
    // Assuming you have 32 buckets and 8 Vec4 elements, each Vec4 will hold values from 4 buckets.
    let max_value = buckets.iter().cloned().fold(f32::MIN, f32::max);
    let mut normalized_buckets = [Vec4::ZERO; 8];

    for (i, &value) in buckets.iter().enumerate() {
        let vec_index = i / 4; // This will give you indices 0 to 7 for 32 buckets
        let component_index = i % 4; // This will give you component indices 0 to 3
        if vec_index < normalized_buckets.len() {
            // Normalize and assign the bucket value to the corresponding Vec4 component
            normalized_buckets[vec_index][component_index] = value / max_value;
        }
    }

    normalized_buckets
}

// The smooth function as you provided
fn smooth(
    buffer: &mut Vec<f32>,
    smoothing: u32,
    smoothing_size: u32,
) {
    for _ in 0..smoothing {
        for i in 0..buffer.len() - smoothing_size as usize {
            // Reduce smoothing for higher freqs more aggressively
            let percentage: f32 = i as f32 / buffer.len() as f32;
            // This is the change: using a higher power to decrease the smoothing size more for higher frequencies
            let adjusted_smoothing_size = (smoothing_size as f32 * (1.0 - percentage.powf(3.0))).max(1.0) as u32;

            let mut y = 0.0;
            for x in 0..adjusted_smoothing_size as usize {
                y += buffer[i + x];
            }
            buffer[i] = y / adjusted_smoothing_size as f32;
        }
    }
}
