use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyConfig {
    pub mic_mode: bool,
    pub host: Option<String>,
    pub device: Option<String>,
    pub sample_rate: i32,
    pub frequency_min: f32,
    pub frequency_max: f32,
    pub smoothing: u32,
    pub smoothing_size: u32,
    pub interpolation_factor: f32,
}
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            mic_mode: false,
            host: None,
            device: None,
            sample_rate: 48000,
            frequency_min: 20.,
            frequency_max: 20_000.,
            smoothing: 2,
            smoothing_size: 4,
            interpolation_factor: 0.6,
        }
    }
}
