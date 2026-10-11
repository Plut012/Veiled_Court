use serde::{Deserialize, Serialize};

/// Directory holding KataGo configs
/// Checks KITSUNE_CONFIG_DIR env var first (for dev mode), falls back to "configs/"
pub fn config_dir() -> String {
    std::env::var("KITSUNE_CONFIG_DIR").unwrap_or_else(|_| "configs".to_string())
}

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
    /// Minimum time a spirit takes before answering a move — the pause is part
    /// of its character. KataGo's own search time counts toward it.
    pub fn min_think_time(&self) -> std::time::Duration {
        let secs = match self {
            Spirit::PrayingMantis => 6.0, // stillness, then not
            Spirit::Dragon => 5.0,
            Spirit::Crane => 4.5,
            Spirit::Eagle => 4.0,
            Spirit::Spider => 4.0,
            Spirit::Jaguar => 3.5,
            Spirit::Lion => 3.0,
            Spirit::Crow => 3.0,
            Spirit::MantisShrimp => 2.0, // its search is long already
        };
        std::time::Duration::from_secs_f64(secs)
    }

    /// Get the config file path for this spirit (see `config_dir`)
    pub fn config_file(&self) -> String {
        let config_dir = config_dir();

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
