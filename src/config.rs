use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub hotkeys: Hotkeys,
    pub settings: Settings,
    pub discord_access_token: Option<String>,
    pub discord_client_id: Option<String>,
    pub discord_client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkeys {
    pub toggle_overlay: String,
    pub nav_up: String,
    pub nav_down: String,
    pub vol_decrease: String,
    pub vol_increase: String,
    pub fast_modifier: String,
    pub mute: String,
    pub jump_top: String,
    pub jump_bottom: String,
    // pub pin: String,
    pub accordion_open: String,
    pub accordion_close: String,
    pub ptt_mode_toggle: String,
    pub ptt_mic_hold: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub normal_step_percent: f32,
    pub fast_step_percent: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkeys: Hotkeys {
                toggle_overlay: "BackQuote".to_string(),
                nav_up: "K".to_string(),
                nav_down: "J".to_string(),
                vol_decrease: "H".to_string(),
                vol_increase: "L".to_string(),
                fast_modifier: "LeftShift".to_string(),
                mute: "M".to_string(),
                jump_top: "GG".to_string(),
                jump_bottom: "G".to_string(),
                // pub pin: String,
                accordion_open: "Enter".to_string(),
                accordion_close: "Escape".to_string(),
                ptt_mode_toggle: "T".to_string(),
                ptt_mic_hold: "V".to_string(),
            },
            settings: Settings {
                normal_step_percent: 2.0,
                fast_step_percent: 10.0,
            },
            discord_access_token: None,
            discord_client_id: None,
            discord_client_secret: None,
        }
    }
}

impl AppConfig {
    fn get_config_path() -> Option<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "oto", "oto") {
            let config_dir = proj_dirs.config_dir();
            if !config_dir.exists() {
                let _ = fs::create_dir_all(config_dir);
            }
            Some(config_dir.join("config.json"))
        } else {
            None
        }
    }

    pub fn load_or_create() -> Self {
        if let Some(path) = Self::get_config_path() {
            if path.exists() {
                if let Ok(config_str) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str(&config_str) {
                        return config;
                    }
                }
            }
            
            let default_config = Self::default();
            if let Ok(json_str) = serde_json::to_string_pretty(&default_config) {
                let _ = fs::write(path, json_str);
            }
            default_config
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        if let Some(path) = Self::get_config_path() {
            if let Ok(json_str) = serde_json::to_string_pretty(self) {
                if let Err(e) = fs::write(&path, json_str) {
                    eprintln!("Failed to save config to {:?}: {}", path, e);
                } else {
                    println!("Settings successfully saved to {:?}", path);
                }
            }
        }
    }
}