use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rustfft::{num_complex::Complex, FftPlanner};

use crate::audio_capture::AudioReceiver;
use crate::ARRAY_UNIFORM_SIZE;
use crate::NUM_BUCKETS;

use crate::bar_material::AudioMaterial;
use crate::circle_material::CircleMaterial;
use crate::polygon_material::PolygonMaterial;
use crate::VisualizationType;

#[derive(Resource)]
pub struct AudioVisualizerState {
    previous_buckets: Vec<f32>,
}

impl AudioVisualizerState {
    pub fn new(num_buckets: usize) -> Self {
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
pub fn audio_event_system(
    audio_receiver: Res<AudioReceiver>,
    mut bar_material: ResMut<Assets<AudioMaterial>>,
    mut circle_material: ResMut<Assets<CircleMaterial>>,
    mut polygon_material: ResMut<Assets<PolygonMaterial>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut visualizer_state: ResMut<AudioVisualizerState>,
    visualization_type: Res<VisualizationType>,
) {
    if let Some(window) = primary_window.iter().next() {
        let window_size = Vec2::new(window.width(), window.height());

        if window_size.x > 0.0 && window_size.y > 0.0 {
            if let Ok(audio_event) = audio_receiver.receiver.lock().unwrap().try_recv() {
                //println!("{:#?}", audio_event);
                let mut fft_planner = FftPlanner::new();
                let fft = fft_planner.plan_fft_forward(2048);

                // Convert audio samples to complex numbers for FFT
                // Assuming audio_event.0 is a Vec<Vec4>, where Vec4 is a type with four f32 fields
                let mut input: Vec<Complex<f32>> = audio_event
                    .0
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
                    let mut buckets = bucketize_fft_to_ranges(&input, NUM_BUCKETS, 40000);

                    // Apply smoothing to the buckets
                    let smoothing = 2;
                    let smoothing_size = 4;
                    smooth(&mut buckets, smoothing, smoothing_size);

                    // Animate bucket transitions
                    let interpolation_factor = 0.5; // Adjust this value as needed
                    let animated_buckets =
                        visualizer_state.animate_buckets(&buckets, interpolation_factor);

                    // Normalize animated buckets for visualization
                    let normalized_buckets = normalize_buckets(&animated_buckets);

                    match *visualization_type {
                        VisualizationType::Bar => {
                            // Update the material properties
                            for (_, material) in bar_material.iter_mut() {
                                material.normalized_data = normalized_buckets;
                                material.viewport_width = window_size.x;
                                material.viewport_height = window_size.y;
                            }
                        }
                        VisualizationType::Circle => {
                            // Update the material properties
                            for (_, material) in circle_material.iter_mut() {
                                material.normalized_data = normalized_buckets;
                                material.viewport_width = window_size.x;
                                material.viewport_height = window_size.y;
                            }
                        }
                        VisualizationType::Polygon => {
                            for (_, material) in polygon_material.iter_mut() {
                                material.normalized_data = normalized_buckets;
                                material.viewport_width = window_size.x;
                                material.viewport_height = window_size.y;
                            }
                        }
                    }
                }
            }
        }
    }
}
fn apply_hann_window(input: &mut Vec<Complex<f32>>) {
    let len = input.len();
    for (i, sample) in input.iter_mut().enumerate() {
        let window_value =
            0.5 * (1.0 - Float::cos(2.0 * std::f32::consts::PI * i as f32 / (len - 1) as f32));
        *sample *= Complex::new(window_value, 0.0);
    }
}
use rustfft::num_traits::Float; // Import the Float trait
fn bucketize_fft_to_ranges(
    input: &[Complex<f32>],
    num_buckets: usize,
    sample_rate: usize,
) -> Vec<f32> {
    let mut buckets = vec![0f32; num_buckets];
    let half_len = input.len() / 2;

    let min_log_freq = 20f32.log2(); // Log2 of 20 Hz
    let max_log_freq = (sample_rate as f32 / 2.0).log2(); // Log2 of Nyquist frequency

    // Iterate over the first half of the FFT output
    for (i, bin) in input.iter().enumerate().take(half_len) {
        let freq = i as f32 * sample_rate as f32 / input.len() as f32; // Frequency of the FFT bin
        let log_freq = freq.log2();

        // Calculate the bucket index based on the logarithmic frequency
        let bucket_index = ((log_freq - min_log_freq) / (max_log_freq - min_log_freq)
            * num_buckets as f32)
            .floor() as usize;

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

fn normalize_buckets(buckets: &[f32]) -> [Vec4; ARRAY_UNIFORM_SIZE] {
    // Assuming you have 32 buckets and 8 Vec4 elements, each Vec4 will hold values from 4 buckets.
    let max_value = buckets.iter().cloned().fold(f32::MIN, f32::max);
    let mut normalized_buckets = [Vec4::ZERO; ARRAY_UNIFORM_SIZE];

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

fn smooth(buffer: &mut Vec<f32>, smoothing: u32, smoothing_size: u32) {
    for _ in 0..smoothing {
        let temp_buffer = buffer.clone();

        for i in 0..buffer.len() {
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;
            for j in i.saturating_sub(smoothing_size as usize)..=i + smoothing_size as usize {
                if j < buffer.len() {
                    // Apply a weight that decreases with distance from the current index
                    let distance = (j as isize - i as isize).abs() as f32;
                    let weight = 1.0 / (1.0 + distance); // You can adjust the formula for weight as needed
                    weighted_sum += temp_buffer[j] * weight;
                    weight_sum += weight;
                }
            }
            buffer[i] = weighted_sum / weight_sum;
        }
    }
}

//// The smooth function as you provided
//fn smooth(buffer: &mut Vec<f32>, smoothing: u32, smoothing_size: u32) {
//    for _ in 0..smoothing {
//        for i in 0..buffer.len() - smoothing_size as usize {
//            // Reduce smoothing for higher freqs more aggressively
//            let percentage: f32 = i as f32 / buffer.len() as f32;
//            // This is the change: using a higher power to decrease the smoothing size more for higher frequencies
//            let adjusted_smoothing_size =
//                (smoothing_size as f32 * (1.0 - percentage.powf(1.8))).max(1.0) as u32;
//
//            let mut y = 0.0;
//            for x in 0..adjusted_smoothing_size as usize {
//                y += buffer[i + x];
//            }
//            buffer[i] = y / adjusted_smoothing_size as f32;
//        }
//    }
//}
