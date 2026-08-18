use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindingsConfig {
    pub quit: String,
    pub help: String,
    pub toggle_config: String,
    pub theme_picker: String,
    pub add_feed: String,
    pub export_opml: String,
    pub delete_item: String,
    pub toggle_zen: String,
    pub search: String,
    pub refresh_current: String,
    pub refresh_all: String,
    pub toggle_read: String,
    pub mark_all_read: String,
    pub toggle_star: String,
    pub open_browser: String,
    pub copy_url: String,
    pub focus_next_pane: String,
    pub focus_prev_pane: String,
    pub focus_sidebar: String,
    pub focus_article_list: String,
    pub focus_reader: String,
    pub nav_down: String,
    pub nav_up: String,
    pub page_down: String,
    pub page_up: String,
    pub space_advance: String,
    pub jump_top: String,
    pub jump_bottom: String,
    pub select_enter: String,
    pub resize_sidebar_dec: String,
    pub resize_sidebar_inc: String,
    pub resize_article_dec: String,
    pub resize_article_inc: String,
    pub resize_reader_inc: String,
    pub resize_reader_dec: String,
    pub reset_layout: String,
}

impl Default for KeyBindingsConfig {
    fn default() -> Self {
        Self {
            quit: "q, ctrl+c".to_string(),
            help: "?, f1".to_string(),
            toggle_config: "/".to_string(),
            theme_picker: "t, T".to_string(),
            add_feed: "a".to_string(),
            export_opml: "e".to_string(),
            delete_item: "d".to_string(),
            toggle_zen: "f, z".to_string(),
            search: "ctrl+f, ctrl+s".to_string(),
            refresh_current: "r".to_string(),
            refresh_all: "R".to_string(),
            toggle_read: "m".to_string(),
            mark_all_read: "M".to_string(),
            toggle_star: "s".to_string(),
            open_browser: "o".to_string(),
            copy_url: "y".to_string(),
            focus_next_pane: "tab, l, right".to_string(),
            focus_prev_pane: "backtab, h, left".to_string(),
            focus_sidebar: "1".to_string(),
            focus_article_list: "2".to_string(),
            focus_reader: "3".to_string(),
            nav_down: "j, down".to_string(),
            nav_up: "k, up".to_string(),
            page_down: "pagedown".to_string(),
            page_up: "pageup".to_string(),
            space_advance: "space".to_string(),
            jump_top: "g, home".to_string(),
            jump_bottom: "G, end".to_string(),
            select_enter: "enter".to_string(),
            resize_sidebar_dec: "<".to_string(),
            resize_sidebar_inc: ">".to_string(),
            resize_article_dec: "[".to_string(),
            resize_article_inc: "]".to_string(),
            resize_reader_inc: "+".to_string(),
            resize_reader_dec: "-".to_string(),
            reset_layout: "=".to_string(),
        }
    }
}

impl KeyBindingsConfig {
    pub fn matches(&self, event: &KeyEvent, binding_str: &str) -> bool {
        key_matches(event, binding_str)
    }
}

pub fn key_matches(event: &KeyEvent, binding: &str) -> bool {
    for part in binding.split(',') {
        let trimmed = part.trim();
        if single_key_matches(event, trimmed) {
            return true;
        }
    }
    false
}

fn single_key_matches(event: &KeyEvent, spec: &str) -> bool {
    if spec.is_empty() {
        return false;
    }
    let lower = spec.to_lowercase();
    let has_ctrl = lower.contains("ctrl+") || lower.contains("control+");
    let has_alt = lower.contains("alt+");
    let has_shift = lower.contains("shift+");

    if has_ctrl && !event.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }
    if !has_ctrl && event.modifiers.contains(KeyModifiers::CONTROL) {
        return false;
    }
    if has_alt && !event.modifiers.contains(KeyModifiers::ALT) {
        return false;
    }
    if !has_alt && event.modifiers.contains(KeyModifiers::ALT) {
        return false;
    }

    let key_name = lower
        .replace("ctrl+", "")
        .replace("control+", "")
        .replace("alt+", "")
        .replace("shift+", "");

    match key_name.as_str() {
        "tab" => event.code == KeyCode::Tab,
        "backtab" | "shift+tab" => {
            event.code == KeyCode::BackTab
                || (event.code == KeyCode::Tab && event.modifiers.contains(KeyModifiers::SHIFT))
        }
        "enter" | "return" => event.code == KeyCode::Enter,
        "esc" | "escape" => event.code == KeyCode::Esc,
        "space" => event.code == KeyCode::Char(' '),
        "backspace" => event.code == KeyCode::Backspace,
        "delete" => event.code == KeyCode::Delete,
        "left" => event.code == KeyCode::Left,
        "right" => event.code == KeyCode::Right,
        "up" => event.code == KeyCode::Up,
        "down" => event.code == KeyCode::Down,
        "pagedown" | "pgdn" => event.code == KeyCode::PageDown,
        "pageup" | "pgup" => event.code == KeyCode::PageUp,
        "home" => event.code == KeyCode::Home,
        "end" => event.code == KeyCode::End,
        "f1" => event.code == KeyCode::F(1),
        "f2" => event.code == KeyCode::F(2),
        "f3" => event.code == KeyCode::F(3),
        "f4" => event.code == KeyCode::F(4),
        "f5" => event.code == KeyCode::F(5),
        "f6" => event.code == KeyCode::F(6),
        "f7" => event.code == KeyCode::F(7),
        "f8" => event.code == KeyCode::F(8),
        "f9" => event.code == KeyCode::F(9),
        "f10" => event.code == KeyCode::F(10),
        "f11" => event.code == KeyCode::F(11),
        "f12" => event.code == KeyCode::F(12),
        single if single.chars().count() == 1 => {
            let spec_char = spec.chars().last().unwrap();
            match event.code {
                KeyCode::Char(c) => {
                    if spec_char.is_uppercase() {
                        c == spec_char
                            || (c.to_ascii_uppercase() == spec_char
                                && event.modifiers.contains(KeyModifiers::SHIFT))
                    } else if has_shift {
                        c.to_ascii_lowercase() == spec_char.to_ascii_lowercase()
                            && event.modifiers.contains(KeyModifiers::SHIFT)
                    } else {
                        c == spec_char
                    }
                }
                _ => false,
            }
        }
        _ => false,
    }
}

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

    /// Horizontal padding inside every pane, in cells (0-6).
    #[serde(default = "default_padding")]
    pub padding: u16,

    /// Gap between article cards, in *half* rows (0-6). 0 keeps the cards
    /// flush; 1 averages half a row by blanking every second gap.
    #[serde(default)]
    pub article_spacing: u16,

    /// Whether the compact shortcut hints are drawn in the status bar.
    /// Off by default — the status line stays clean until asked. Toggled live
    /// with `??`, or from the settings menu.
    #[serde(default)]
    pub show_help_hints: bool,

    #[serde(default)]
    pub keybindings: KeyBindingsConfig,
}

fn default_padding() -> u16 {
    1
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "RataRSS Dark".to_string(),
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
            padding: default_padding(),
            article_spacing: 0,
            show_help_hints: false,
            keybindings: KeyBindingsConfig::default(),
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
