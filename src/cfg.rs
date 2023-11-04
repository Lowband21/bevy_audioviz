use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyConfig {
    pub host: Option<String>,
    pub device: Option<String>,
}
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            host: None,
            device: None,
        }
    }
}
