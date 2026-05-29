use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    // Config properties to persist to disk (e.g. library path, default port)
}

#[allow(dead_code)]
pub fn load_config() -> Config {
    // Load config from directories crate path
    Config::default()
}
