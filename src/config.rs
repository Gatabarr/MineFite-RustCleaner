use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub save_path: String,
    pub add_spaces: bool,
    pub remove_non_rp: bool,
    pub use_base_list: bool,
    pub excluded_words: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_path: "C:/".to_string(),
            add_spaces: false,
            remove_non_rp: false,
            use_base_list: true,
            excluded_words: vec!["[Чат фракции]".to_string()],
        }
    }
}

pub fn get_app_dir() -> PathBuf {
    let mut path = if let Ok(app_data) = std::env::var("APPDATA") {
        PathBuf::from(app_data).join("LogCleaner")
    } else {
        PathBuf::from(".")
    };
    let _ = fs::create_dir_all(&path);
    path
}

pub fn load_config() -> Config {
    let path = get_app_dir().join("config.json");
    if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) {
    let path = get_app_dir().join("config.json");
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, content);
    }
}