use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    
    // Layout pane percentages (must sum to 100 or close)
    pub sidebar_ratio: u16,
    pub article_list_ratio: u16,
    pub reader_ratio: u16,
    
    // Behavior
    pub refresh_interval_minutes: u64,
    pub auto_refresh_on_startup: bool,
    pub mark_read_on_open: bool,
    pub open_browser_command: Option<String>,
    pub max_articles_per_feed: usize,
    pub show_icons: bool,
    pub wrap_article_text: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "NetNewsWire Dark".to_string(),
            sidebar_ratio: 22,
            article_list_ratio: 33,
            reader_ratio: 45,
            refresh_interval_minutes: 15,
            auto_refresh_on_startup: true,
            mark_read_on_open: true,
            open_browser_command: None,
            max_articles_per_feed: 200,
            show_icons: true,
            wrap_article_text: true,
        }
    }
}

impl AppConfig {
    pub fn get_config_dir() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("com", "ratarss", "ratarss") {
            dirs.config_dir().to_path_buf()
        } else {
            PathBuf::from("./.config/ratarss")
        }
    }

    pub fn get_data_dir() -> PathBuf {
        if let Some(dirs) = ProjectDirs::from("com", "ratarss", "ratarss") {
            dirs.data_dir().to_path_buf()
        } else {
            PathBuf::from("./.data/ratarss")
        }
    }

    pub fn load() -> Self {
        let config_dir = Self::get_config_dir();
        let config_file = config_dir.join("config.toml");
        
        if config_file.exists() {
            if let Ok(content) = fs::read_to_string(&config_file) {
                if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                    return config;
                }
            }
        }
        
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let config_dir = Self::get_config_dir();
        fs::create_dir_all(&config_dir)?;
        let config_file = config_dir.join("config.toml");
        let toml_str = toml::to_string_pretty(self).unwrap_or_default();
        fs::write(config_file, toml_str)?;
        Ok(())
    }
}
