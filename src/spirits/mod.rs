use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Spirit {
    Dragon,
    MantisShrimp,
    Crane,
    Spider,
    Eagle,
    Lion,
    PrayingMantis,
    Jaguar,
    Crow,
}

impl Spirit {
    /// Get the config file path for this spirit
    /// Checks ANIMAL_GO_CONFIG_DIR env var first (for dev mode), falls back to "configs/"
    pub fn config_file(&self) -> String {
        let config_dir = std::env::var("ANIMAL_GO_CONFIG_DIR")
            .unwrap_or_else(|_| "configs".to_string());

        let filename = match self {
            Spirit::Dragon => "dragon.cfg",
            Spirit::MantisShrimp => "mantis_shrimp.cfg",
            Spirit::Crane => "crane.cfg",
            Spirit::Spider => "spider.cfg",
            Spirit::Eagle => "eagle.cfg",
            Spirit::Lion => "lion.cfg",
            Spirit::PrayingMantis => "praying_mantis.cfg",
            Spirit::Jaguar => "jaguar.cfg",
            Spirit::Crow => "crow.cfg",
        };

        format!("{}/{}", config_dir, filename)
    }

    pub fn from_string(s: &str) -> Option<Spirit> {
        match s.to_lowercase().as_str() {
            "dragon" => Some(Spirit::Dragon),
            "mantis_shrimp" => Some(Spirit::MantisShrimp),
            "crane" => Some(Spirit::Crane),
            "spider" => Some(Spirit::Spider),
            "eagle" => Some(Spirit::Eagle),
            "lion" => Some(Spirit::Lion),
            "praying_mantis" => Some(Spirit::PrayingMantis),
            "jaguar" => Some(Spirit::Jaguar),
            "crow" => Some(Spirit::Crow),
            _ => None,
        }
    }

    pub fn palette(&self) -> Palette {
        match self {
            Spirit::Dragon => Palette {
                board_primary: "#0D1F1A",
                board_secondary: "#2E6B55",
                accent: "#C9A84C",
            },
            Spirit::MantisShrimp => Palette {
                board_primary: "#080B10",
                board_secondary: "#0D4D5E",
                accent: "#9B5DE5",
            },
            Spirit::Crane => Palette {
                board_primary: "#E8E4DC",
                board_secondary: "#4A4A52",
                accent: "#A8B8C8",
            },
            Spirit::Spider => Palette {
                board_primary: "#0F1410",
                board_secondary: "#3A3F38",
                accent: "#C17D2A",
            },
            Spirit::Eagle => Palette {
                board_primary: "#1A2535",
                board_secondary: "#6B7A8D",
                accent: "#D4A843",
            },
            Spirit::Lion => Palette {
                board_primary: "#2A1E0F",
                board_secondary: "#8B6914",
                accent: "#E8A020",
            },
            Spirit::PrayingMantis => Palette {
                board_primary: "#0C0F0C",
                board_secondary: "#1A4030",
                accent: "#8B1A1A",
            },
            Spirit::Jaguar => Palette {
                board_primary: "#1C1508",
                board_secondary: "#5C3D1A",
                accent: "#C9A870",
            },
            Spirit::Crow => Palette {
                board_primary: "#12141A",
                board_secondary: "#2E3340",
                accent: "#E8EEF4",
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Palette {
    pub board_primary: &'static str,
    pub board_secondary: &'static str,
    pub accent: &'static str,
}
