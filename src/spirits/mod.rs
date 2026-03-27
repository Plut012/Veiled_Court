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
    pub fn config_file(&self) -> &'static str {
        match self {
            Spirit::Dragon => "configs/dragon.cfg",
            Spirit::MantisShrimp => "configs/mantis_shrimp.cfg",
            Spirit::Crane => "configs/crane.cfg",
            Spirit::Spider => "configs/spider.cfg",
            Spirit::Eagle => "configs/eagle.cfg",
            Spirit::Lion => "configs/lion.cfg",
            Spirit::PrayingMantis => "configs/praying_mantis.cfg",
            Spirit::Jaguar => "configs/jaguar.cfg",
            Spirit::Crow => "configs/crow.cfg",
        }
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
