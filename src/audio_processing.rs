use bevy::prelude::*;
use bevy::utils::Duration;
use bevy::window::PrimaryWindow;

use crate::audio_capture::AudioReceiver;
use crate::ARRAY_UNIFORM_SIZE;
use crate::NUM_BUCKETS;

use crate::materials::{
    BarMaterial, CircleSplitMaterial, PolygonMaterial, StringMaterial, WaveMaterial,
};
use crate::VisualizationType;
use crate::{CfgResource, MyConfig};
use spectrum_analyzer::windows::hann_window; // Import the window function

use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit, FrequencySpectrum};

#[derive(Resource)]
pub struct AudioVisualizerState {
    previous_buckets_left: Vec<f32>,
    previous_buckets_right: Vec<f32>,
}

impl AudioVisualizerState {
    pub fn new(num_buckets: usize) -> Self {
        AudioVisualizerState {
            previous_buckets_left: vec![0.0; num_buckets],
            previous_buckets_right: vec![0.0; num_buckets],
        }
    }

    fn animate_buckets(
        &mut self,
        current_buckets: &[f32],
        interpolation_factor: f32,
        is_left_channel: bool,
        config: &MyConfig,
    ) -> Vec<f32> {
        let previous_buckets = if is_left_channel {
            &mut self.previous_buckets_left
        } else {
            &mut self.previous_buckets_right
        };

        let mut animated_buckets = Vec::with_capacity(current_buckets.len());

        if current_buckets.iter().fold(0.0, |sum, &val| sum + val) <= config.gate_threshold {
            animated_buckets = vec![0.0; current_buckets.len()];
        } else {
            for (&current, previous) in current_buckets.iter().zip(previous_buckets.iter_mut()) {
                // Interpolate between the previous bucket value and the current one
                let interpolated_value = *previous + (current - *previous) * interpolation_factor;
                animated_buckets.push(interpolated_value);
                // Update the previous value for the next frame
                *previous = interpolated_value;
            }
        }

        animated_buckets
    }
}

fn is_power_of_two(number: usize) -> bool {
    number != 0 && (number & (number - 1)) == 0
}

// Entry function for the audio event system
pub fn audio_event_system(
    audio_receiver: Res<AudioReceiver>,
    mut bar_material: ResMut<Assets<BarMaterial>>,
    mut string_material: ResMut<Assets<StringMaterial>>,
    mut circle_split_material: ResMut<Assets<CircleSplitMaterial>>,
    mut polygon_material: ResMut<Assets<PolygonMaterial>>,
    mut wave_material: ResMut<Assets<WaveMaterial>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut visualizer_state: ResMut<AudioVisualizerState>,
    visualization_type: Res<VisualizationType>,
    config: Res<CfgResource>,
) {
    if let Some(window) = primary_window.iter().next() {
        let window_size = Vec2::new(window.width(), window.height());

        if window_size.x > 0.0 && window_size.y > 0.0 {
            if let Ok(audio_event) = audio_receiver
                .receiver
                .lock()
                .unwrap()
                .recv_timeout(Duration::new(0, 100))
            {
                // Flatten the audio samples into a single Vec<f32>
                let left_samples = audio_event.left.to_vec();
                let right_samples = audio_event.right.to_vec();

                let left_buckets =
                    samples_to_buckets(config.0.clone(), left_samples, &mut visualizer_state, true)
                        .unwrap();
                let right_buckets = samples_to_buckets(
                    config.0.clone(),
                    right_samples,
                    &mut visualizer_state,
                    false,
                )
                .unwrap();
                //println!("{:#?}", right_buckets);

                // Update visualizer materials with normalized buckets
                update_visualizer_materials(
                    &left_buckets,
                    &right_buckets,
                    &window_size,
                    &visualization_type,
                    &mut bar_material,
                    &mut string_material,
                    &mut circle_split_material,
                    &mut polygon_material,
                    &mut wave_material,
                );
            }
        }
    }
}

fn samples_to_buckets(
    config: MyConfig,
    mut samples: Vec<f32>,
    visualizer_state: &mut ResMut<AudioVisualizerState>,
    is_left_channel: bool,
) -> Option<[Vec4; ARRAY_UNIFORM_SIZE]> {
    // Apply a window function to the samples
    samples = hann_window(&samples);

    // Ensure the sample length is a power of two, pad with zeroes if necessary
    if !is_power_of_two(samples.len()) {
        let next_power_of_two = samples.len().next_power_of_two();
        samples.resize(next_power_of_two, 0.0);
    }

    // Compute the frequency spectrum using the spectrum_analyzer crate
    let spectrum_result = samples_fft_to_spectrum(
        &samples,                                                          // windowed samples
        48000, // Replace with the actual sample rate of your audio
        FrequencyLimit::Range(config.frequency_min, config.frequency_max), // Adjust the frequency range as needed
        None, //Some(&divide_by_N_sqrt),             // Normalization function
    );

    if let Ok(spectrum) = spectrum_result {
        // Transform the frequency spectrum into buckets for visualization
        let mut buckets = transform_spectrum_to_buckets(&spectrum, NUM_BUCKETS);

        // Apply smoothing to the buckets
        let smoothing = config.smoothing;
        let smoothing_size = config.smoothing_size;
        smooth(&mut buckets, smoothing, smoothing_size);

        //let amplification_factor = 1.5;
        //amplify_differences(&mut buckets, amplification_factor);

        // add a gate
        gate(&mut buckets, config.gate_threshold);

        // Animate the transition of buckets
        let interpolation_factor = config.interpolation_factor; // Adjust this value as needed
        let animated_buckets = visualizer_state.animate_buckets(
            &buckets,
            interpolation_factor,
            is_left_channel,
            &config,
        );

        // Normalize the animated buckets for visualization
        let normalized_buckets = normalize_buckets(&animated_buckets);
        Some(normalized_buckets)
    } else {
        println!("Spectrum analysis failed");
        None
    }
}

fn gate(buckets: &mut Vec<f32>, gate_threshold: f32) {
    let len = buckets.len();
    let mut count = 0;
    let mut max = f32::MIN;

    // Find the count of elements below the threshold and the maximum value
    for freq in buckets.iter() {
        if *freq < gate_threshold {
            count += 1;
        }
        if *freq > max {
            max = *freq;
        }
    }

    // If the count is greater than the specified threshold, modify the vector
    if count > len - (len / 8) {
        let threshold = max - gate_threshold;
        for freq in buckets.iter_mut() {
            if *freq > threshold || *freq < gate_threshold {
                *freq = 0.0;
            }
        }
    }
}

fn update_visualizer_materials(
    left_buckets: &[Vec4; ARRAY_UNIFORM_SIZE],
    right_buckets: &[Vec4; ARRAY_UNIFORM_SIZE],
    window_size: &Vec2,
    visualization_type: &VisualizationType,
    bar_material: &mut ResMut<Assets<BarMaterial>>,
    string_material: &mut ResMut<Assets<StringMaterial>>,
    circle_split_material: &mut ResMut<Assets<CircleSplitMaterial>>,
    polygon_material: &mut ResMut<Assets<PolygonMaterial>>,
    wave_material: &mut ResMut<Assets<WaveMaterial>>,
) {
    let mono_buckets = if needs_mono(visualization_type) {
        mix_mono_channels(left_buckets, right_buckets)
    } else {
        *left_buckets
    };

    match visualization_type {
        VisualizationType::Bar => {
            for (_, material) in bar_material.iter_mut() {
                material.normalized_data = mono_buckets;
                material.viewport_width = window_size.x;
                material.viewport_height = window_size.y;
            }
        }
        VisualizationType::String => {
            for (_, material) in string_material.iter_mut() {
                material.left_data = *left_buckets;
                material.right_data = *right_buckets;
                material.viewport_width = window_size.x;
                material.viewport_height = window_size.y;
            }
        }
        VisualizationType::CircleSplit => {
            for (_, material) in circle_split_material.iter_mut() {
                material.left_data = *left_buckets;
                material.right_data = *right_buckets;
                material.viewport_width = window_size.x;
                material.viewport_height = window_size.y;
            }
        }
        VisualizationType::Wave => {
            for (_, material) in wave_material.iter_mut() {
                material.left_data = *left_buckets;
                material.right_data = *right_buckets;
                material.viewport_width = window_size.x;
                material.viewport_height = window_size.y;
            }
        }
        VisualizationType::Polygon => {
            for (_, material) in polygon_material.iter_mut() {
                material.normalized_data = mono_buckets;
                material.viewport_width = window_size.x;
                material.viewport_height = window_size.y;
            }
        }
    }
}

fn mix_mono_channels(
    left_buckets: &[Vec4; ARRAY_UNIFORM_SIZE],
    right_buckets: &[Vec4; ARRAY_UNIFORM_SIZE],
) -> [Vec4; ARRAY_UNIFORM_SIZE] {
    let mut mixed_buckets = [Vec4::ZERO; ARRAY_UNIFORM_SIZE];
    for (i, (left, right)) in left_buckets.iter().zip(right_buckets.iter()).enumerate() {
        mixed_buckets[i] = (*left + *right) * 0.5;
    }
    mixed_buckets
}

fn needs_mono(visualization_type: &VisualizationType) -> bool {
    matches!(
        visualization_type,
        VisualizationType::Bar | VisualizationType::Polygon
    )
}

fn transform_spectrum_to_buckets(spectrum: &FrequencySpectrum, num_buckets: usize) -> Vec<f32> {
    let mut buckets = vec![0f32; num_buckets];

    // Determine the logarithmic scale factor based on the max frequency
    let max_frequency = spectrum.max_fr().val();
    let min_frequency = spectrum.min_fr().val(); // Assume there's a min_fr() or set a reasonable minimum frequency like 20.0
    let log_min_frequency = min_frequency.ln();
    let log_max_frequency = max_frequency.ln();

    for &(frequency, value) in spectrum.data() {
        let freq_val = frequency.val();

        // Skip frequencies lower than the minimum frequency (e.g., 20 Hz)
        if freq_val < min_frequency {
            continue;
        }

        // Calculate the bucket index on a logarithmic scale
        let log_freq = freq_val.ln();
        let scale = (log_freq - log_min_frequency) / (log_max_frequency - log_min_frequency);
        let bucket_index = (scale * (num_buckets as f32 - 1.0)) as usize;

        // Add the magnitude to the appropriate bucket
        buckets[bucket_index] += value.val();
    }

    buckets
}

fn normalize_buckets(buckets: &[f32]) -> [Vec4; ARRAY_UNIFORM_SIZE] {
    let max_value = buckets.iter().cloned().fold(f32::MIN, f32::max);
    let mut normalized_buckets = [Vec4::ZERO; ARRAY_UNIFORM_SIZE];

    for (i, &value) in buckets.iter().enumerate() {
        let vec_index = i / 4;
        let component_index = i % 4; // This will give you component indices 0 to 3
        if vec_index < normalized_buckets.len() {
            // Normalize and assign the bucket value to the corresponding Vec4 component
            normalized_buckets[vec_index][component_index] = value / max_value;
        }
    }

    normalized_buckets
}

fn smooth(buffer: &mut Vec<f32>, smoothing: u32, smoothing_size: u32) {
    let gaussian_weight =
        |distance: f32| -> f32 { (-distance.powi(2) / (2.0 * smoothing_size as f32)).exp() };

    for _ in 0..smoothing {
        let temp_buffer = buffer.clone();

        for i in 0..buffer.len() {
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;
            for j in i.saturating_sub(smoothing_size as usize)..=i + smoothing_size as usize {
                if j < buffer.len() {
                    let distance = (j as isize - i as isize).abs() as f32;
                    let weight = gaussian_weight(distance);
                    weighted_sum += temp_buffer[j] * weight;
                    weight_sum += weight;
                }
            }
            buffer[i] = weighted_sum / weight_sum;
        }
    }
}
