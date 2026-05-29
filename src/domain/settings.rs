use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub bionic: BionicSettings,
    pub theme: ThemeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BionicSettings {
    pub enabled: bool,
    pub intensity: f32,
}

impl Default for BionicSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub name: String,
    pub font_size: String,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            name: "light".to_string(),
            font_size: "1.1rem".to_string(),
        }
    }
}
