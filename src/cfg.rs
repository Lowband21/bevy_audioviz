use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyConfig {
    pub mic_mode: bool,
    pub host: Option<String>,
    pub device: Option<String>,
}
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            mic_mode: false,
            host: None,
            device: None,
        }
    }
}
