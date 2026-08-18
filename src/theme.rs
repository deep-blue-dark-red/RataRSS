use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub description: String,
    
    // Base Colors
    pub bg: String,
    pub fg: String,
    pub fg_dim: String,
    pub fg_subtle: String,
    pub accent: String,
    pub accent_subtle: String,
    pub selection_bg: String,
    pub selection_fg: String,
    
    // Borders
    pub border_active: String,
    pub border_inactive: String,
    pub border_accent: String,
    
    // Sidebar
    pub sidebar_bg: String,
    pub sidebar_header_fg: String,
    pub sidebar_folder_fg: String,
    pub sidebar_feed_fg: String,
    pub sidebar_unread_badge_bg: String,
    pub sidebar_unread_badge_fg: String,
    
    // Article List
    pub article_list_bg: String,
    pub article_card_bg: String,
    pub article_card_selected_bg: String,
    pub article_card_selected_border: String,
    pub article_title_unread: String,
    pub article_title_read: String,
    pub article_summary_fg: String,
    pub article_meta_fg: String,
    pub article_unread_dot: String,
    pub article_star: String,
    
    // Reader View
    pub reader_bg: String,
    pub reader_header_feed: String,
    pub reader_header_author: String,
    pub reader_title: String,
    pub reader_meta: String,
    pub reader_divider: String,
    pub reader_body: String,
    pub reader_h1: String,
    pub reader_h2: String,
    pub reader_h3: String,
    pub reader_quote_border: String,
    pub reader_quote_fg: String,
    pub reader_code_bg: String,
    pub reader_code_fg: String,
    pub reader_link: String,
    pub reader_link_url: String,
    
    // Status Bar & Modals
    pub status_bar_bg: String,
    pub status_bar_fg: String,
    pub modal_bg: String,
    pub modal_border: String,
    pub error_fg: String,
    pub success_fg: String,
    pub warning_fg: String,
}

pub fn parse_color(color_str: &str) -> Color {
    let s = color_str.trim();
    if s.is_empty() {
        return Color::Reset;
    }
    if s.eq_ignore_ascii_case("reset") {
        return Color::Reset;
    }
    if s.starts_with('#') {
        let hex = &s[1..];
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        } else if hex.len() == 3 {
            let r_char = &hex[0..1];
            let g_char = &hex[1..2];
            let b_char = &hex[2..3];
            let r_str = format!("{r_char}{r_char}");
            let g_str = format!("{g_char}{g_char}");
            let b_str = format!("{b_char}{b_char}");
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&r_str, 16),
                u8::from_str_radix(&g_str, 16),
                u8::from_str_radix(&b_str, 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
    }
    
    match s.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" | "dark_gray" => Color::DarkGray,
        "lightred" | "light_red" => Color::LightRed,
        "lightgreen" | "light_green" => Color::LightGreen,
        "lightyellow" | "light_yellow" => Color::LightYellow,
        "lightblue" | "light_blue" => Color::LightBlue,
        "lightmagenta" | "light_magenta" => Color::LightMagenta,
        "lightcyan" | "light_cyan" => Color::LightCyan,
        "white" => Color::White,
        _ => Color::Reset,
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Theme {

    pub config: ThemeConfig,
    
    pub bg: Color,
    pub fg: Color,
    pub fg_dim: Color,
    pub fg_subtle: Color,
    pub accent: Color,
    pub accent_subtle: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    
    pub border_active: Color,
    pub border_inactive: Color,
    pub border_accent: Color,
    
    pub sidebar_bg: Color,
    pub sidebar_header_fg: Color,
    pub sidebar_folder_fg: Color,
    pub sidebar_feed_fg: Color,
    pub sidebar_unread_badge_bg: Color,
    pub sidebar_unread_badge_fg: Color,
    
    pub article_list_bg: Color,
    pub article_card_bg: Color,
    pub article_card_selected_bg: Color,
    pub article_card_selected_border: Color,
    pub article_title_unread: Color,
    pub article_title_read: Color,
    pub article_summary_fg: Color,
    pub article_meta_fg: Color,
    pub article_unread_dot: Color,
    pub article_star: Color,
    
    pub reader_bg: Color,
    pub reader_header_feed: Color,
    pub reader_header_author: Color,
    pub reader_title: Color,
    pub reader_meta: Color,
    pub reader_divider: Color,
    pub reader_body: Color,
    pub reader_h1: Color,
    pub reader_h2: Color,
    pub reader_h3: Color,
    pub reader_quote_border: Color,
    pub reader_quote_fg: Color,
    pub reader_code_bg: Color,
    pub reader_code_fg: Color,
    pub reader_link: Color,
    pub reader_link_url: Color,
    
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub modal_bg: Color,
    pub modal_border: Color,
    pub error_fg: Color,
    pub success_fg: Color,
    pub warning_fg: Color,
}

impl From<ThemeConfig> for Theme {
    fn from(c: ThemeConfig) -> Self {
        Self {
            bg: parse_color(&c.bg),
            fg: parse_color(&c.fg),
            fg_dim: parse_color(&c.fg_dim),
            fg_subtle: parse_color(&c.fg_subtle),
            accent: parse_color(&c.accent),
            accent_subtle: parse_color(&c.accent_subtle),
            selection_bg: parse_color(&c.selection_bg),
            selection_fg: parse_color(&c.selection_fg),
            
            border_active: parse_color(&c.border_active),
            border_inactive: parse_color(&c.border_inactive),
            border_accent: parse_color(&c.border_accent),
            
            sidebar_bg: parse_color(&c.sidebar_bg),
            sidebar_header_fg: parse_color(&c.sidebar_header_fg),
            sidebar_folder_fg: parse_color(&c.sidebar_folder_fg),
            sidebar_feed_fg: parse_color(&c.sidebar_feed_fg),
            sidebar_unread_badge_bg: parse_color(&c.sidebar_unread_badge_bg),
            sidebar_unread_badge_fg: parse_color(&c.sidebar_unread_badge_fg),
            
            article_list_bg: parse_color(&c.article_list_bg),
            article_card_bg: parse_color(&c.article_card_bg),
            article_card_selected_bg: parse_color(&c.article_card_selected_bg),
            article_card_selected_border: parse_color(&c.article_card_selected_border),
            article_title_unread: parse_color(&c.article_title_unread),
            article_title_read: parse_color(&c.article_title_read),
            article_summary_fg: parse_color(&c.article_summary_fg),
            article_meta_fg: parse_color(&c.article_meta_fg),
            article_unread_dot: parse_color(&c.article_unread_dot),
            article_star: parse_color(&c.article_star),
            
            reader_bg: parse_color(&c.reader_bg),
            reader_header_feed: parse_color(&c.reader_header_feed),
            reader_header_author: parse_color(&c.reader_header_author),
            reader_title: parse_color(&c.reader_title),
            reader_meta: parse_color(&c.reader_meta),
            reader_divider: parse_color(&c.reader_divider),
            reader_body: parse_color(&c.reader_body),
            reader_h1: parse_color(&c.reader_h1),
            reader_h2: parse_color(&c.reader_h2),
            reader_h3: parse_color(&c.reader_h3),
            reader_quote_border: parse_color(&c.reader_quote_border),
            reader_quote_fg: parse_color(&c.reader_quote_fg),
            reader_code_bg: parse_color(&c.reader_code_bg),
            reader_code_fg: parse_color(&c.reader_code_fg),
            reader_link: parse_color(&c.reader_link),
            reader_link_url: parse_color(&c.reader_link_url),
            
            status_bar_bg: parse_color(&c.status_bar_bg),
            status_bar_fg: parse_color(&c.status_bar_fg),
            modal_bg: parse_color(&c.modal_bg),
            modal_border: parse_color(&c.modal_border),
            error_fg: parse_color(&c.error_fg),
            success_fg: parse_color(&c.success_fg),
            warning_fg: parse_color(&c.warning_fg),
            
            config: c,
        }
    }
}

impl Theme {
    pub fn netnewswire_dark() -> Self {
        ThemeConfig {
            name: "NetNewsWire Dark".to_string(),
            description: "Default sleek dark macOS style matching NetNewsWire".to_string(),
            
            bg: "#16181c".to_string(),
            fg: "#eceff4".to_string(),
            fg_dim: "#9aa2b1".to_string(),
            fg_subtle: "#545b68".to_string(),
            accent: "#388bfd".to_string(),
            accent_subtle: "#1f3b64".to_string(),
            selection_bg: "#2b3547".to_string(),
            selection_fg: "#ffffff".to_string(),
            
            border_active: "#388bfd".to_string(),
            border_inactive: "#282c34".to_string(),
            border_accent: "#f0883e".to_string(),
            
            sidebar_bg: "#131418".to_string(),
            sidebar_header_fg: "#7d8590".to_string(),
            sidebar_folder_fg: "#58a6ff".to_string(),
            sidebar_feed_fg: "#d1d5db".to_string(),
            sidebar_unread_badge_bg: "#252b38".to_string(),
            sidebar_unread_badge_fg: "#89a4c7".to_string(),
            
            article_list_bg: "#16181d".to_string(),
            article_card_bg: "#1a1d24".to_string(),
            article_card_selected_bg: "#253347".to_string(),
            article_card_selected_border: "#388bfd".to_string(),
            article_title_unread: "#ffffff".to_string(),
            article_title_read: "#9aa2b1".to_string(),
            article_summary_fg: "#8b949e".to_string(),
            article_meta_fg: "#58a6ff".to_string(),
            article_unread_dot: "#388bfd".to_string(),
            article_star: "#e3b341".to_string(),
            
            reader_bg: "#16181d".to_string(),
            reader_header_feed: "#58a6ff".to_string(),
            reader_header_author: "#8b949e".to_string(),
            reader_title: "#ffffff".to_string(),
            reader_meta: "#6e7681".to_string(),
            reader_divider: "#30363d".to_string(),
            reader_body: "#d8dee9".to_string(),
            reader_h1: "#58a6ff".to_string(),
            reader_h2: "#79c0ff".to_string(),
            reader_h3: "#a5d6ff".to_string(),
            reader_quote_border: "#58a6ff".to_string(),
            reader_quote_fg: "#8b949e".to_string(),
            reader_code_bg: "#0d1117".to_string(),
            reader_code_fg: "#58a6ff".to_string(),
            reader_link: "#79c0ff".to_string(),
            reader_link_url: "#484f58".to_string(),
            
            status_bar_bg: "#0d1117".to_string(),
            status_bar_fg: "#8b949e".to_string(),
            modal_bg: "#161b22".to_string(),
            modal_border: "#388bfd".to_string(),
            error_fg: "#f85149".to_string(),
            success_fg: "#3fb950".to_string(),
            warning_fg: "#d29922".to_string(),
        }.into()
    }

    pub fn netnewswire_light() -> Self {
        ThemeConfig {
            name: "NetNewsWire Light".to_string(),
            description: "Clean Apple macOS light theme".to_string(),
            
            bg: "#ffffff".to_string(),
            fg: "#1d2129".to_string(),
            fg_dim: "#606770".to_string(),
            fg_subtle: "#8d949e".to_string(),
            accent: "#0066cc".to_string(),
            accent_subtle: "#e7f3ff".to_string(),
            selection_bg: "#e4edf7".to_string(),
            selection_fg: "#0055b3".to_string(),
            
            border_active: "#0066cc".to_string(),
            border_inactive: "#e1e4e8".to_string(),
            border_accent: "#f58220".to_string(),
            
            sidebar_bg: "#f5f6f8".to_string(),
            sidebar_header_fg: "#606770".to_string(),
            sidebar_folder_fg: "#0066cc".to_string(),
            sidebar_feed_fg: "#24292e".to_string(),
            sidebar_unread_badge_bg: "#e1e4e8".to_string(),
            sidebar_unread_badge_fg: "#444d56".to_string(),
            
            article_list_bg: "#ffffff".to_string(),
            article_card_bg: "#fafbfc".to_string(),
            article_card_selected_bg: "#e8f2fe".to_string(),
            article_card_selected_border: "#0066cc".to_string(),
            article_title_unread: "#1a1a1a".to_string(),
            article_title_read: "#586069".to_string(),
            article_summary_fg: "#586069".to_string(),
            article_meta_fg: "#0066cc".to_string(),
            article_unread_dot: "#0066cc".to_string(),
            article_star: "#d97706".to_string(),
            
            reader_bg: "#ffffff".to_string(),
            reader_header_feed: "#0066cc".to_string(),
            reader_header_author: "#586069".to_string(),
            reader_title: "#111827".to_string(),
            reader_meta: "#6b7280".to_string(),
            reader_divider: "#e5e7eb".to_string(),
            reader_body: "#1f2937".to_string(),
            reader_h1: "#0066cc".to_string(),
            reader_h2: "#1d4ed8".to_string(),
            reader_h3: "#2563eb".to_string(),
            reader_quote_border: "#0066cc".to_string(),
            reader_quote_fg: "#4b5563".to_string(),
            reader_code_bg: "#f3f4f6".to_string(),
            reader_code_fg: "#0066cc".to_string(),
            reader_link: "#0066cc".to_string(),
            reader_link_url: "#6b7280".to_string(),
            
            status_bar_bg: "#ebeef2".to_string(),
            status_bar_fg: "#4b5563".to_string(),
            modal_bg: "#ffffff".to_string(),
            modal_border: "#0066cc".to_string(),
            error_fg: "#cb2431".to_string(),
            success_fg: "#22863a".to_string(),
            warning_fg: "#b08800".to_string(),
        }.into()
    }

    pub fn catppuccin_mocha() -> Self {
        ThemeConfig {
            name: "Catppuccin Mocha".to_string(),
            description: "Soothing pastel dark theme".to_string(),
            
            bg: "#1e1e2e".to_string(),
            fg: "#cdd6f4".to_string(),
            fg_dim: "#a6adc8".to_string(),
            fg_subtle: "#585b70".to_string(),
            accent: "#89b4fa".to_string(),
            accent_subtle: "#313244".to_string(),
            selection_bg: "#313244".to_string(),
            selection_fg: "#cdd6f4".to_string(),
            
            border_active: "#89b4fa".to_string(),
            border_inactive: "#313244".to_string(),
            border_accent: "#fab387".to_string(),
            
            sidebar_bg: "#181825".to_string(),
            sidebar_header_fg: "#6c7086".to_string(),
            sidebar_folder_fg: "#89b4fa".to_string(),
            sidebar_feed_fg: "#bac2de".to_string(),
            sidebar_unread_badge_bg: "#313244".to_string(),
            sidebar_unread_badge_fg: "#89b4fa".to_string(),
            
            article_list_bg: "#1e1e2e".to_string(),
            article_card_bg: "#181825".to_string(),
            article_card_selected_bg: "#313244".to_string(),
            article_card_selected_border: "#89b4fa".to_string(),
            article_title_unread: "#cdd6f4".to_string(),
            article_title_read: "#7f849c".to_string(),
            article_summary_fg: "#a6adc8".to_string(),
            article_meta_fg: "#89b4fa".to_string(),
            article_unread_dot: "#89b4fa".to_string(),
            article_star: "#f9e2af".to_string(),
            
            reader_bg: "#1e1e2e".to_string(),
            reader_header_feed: "#89b4fa".to_string(),
            reader_header_author: "#a6adc8".to_string(),
            reader_title: "#cdd6f4".to_string(),
            reader_meta: "#6c7086".to_string(),
            reader_divider: "#313244".to_string(),
            reader_body: "#cdd6f4".to_string(),
            reader_h1: "#cba6f7".to_string(),
            reader_h2: "#89b4fa".to_string(),
            reader_h3: "#74c7ec".to_string(),
            reader_quote_border: "#f5c2e7".to_string(),
            reader_quote_fg: "#a6adc8".to_string(),
            reader_code_bg: "#11111b".to_string(),
            reader_code_fg: "#a6e3a1".to_string(),
            reader_link: "#89dceb".to_string(),
            reader_link_url: "#585b70".to_string(),
            
            status_bar_bg: "#11111b".to_string(),
            status_bar_fg: "#a6adc8".to_string(),
            modal_bg: "#181825".to_string(),
            modal_border: "#89b4fa".to_string(),
            error_fg: "#f38ba8".to_string(),
            success_fg: "#a6e3a1".to_string(),
            warning_fg: "#f9e2af".to_string(),
        }.into()
    }

    pub fn tokyo_night() -> Self {
        ThemeConfig {
            name: "Tokyo Night".to_string(),
            description: "Clean Tokyo Night palette with vivid neon accents".to_string(),
            
            bg: "#1a1b26".to_string(),
            fg: "#a9b1d6".to_string(),
            fg_dim: "#787c99".to_string(),
            fg_subtle: "#414868".to_string(),
            accent: "#7aa2f7".to_string(),
            accent_subtle: "#24283b".to_string(),
            selection_bg: "#283457".to_string(),
            selection_fg: "#c0caf5".to_string(),
            
            border_active: "#7aa2f7".to_string(),
            border_inactive: "#292e42".to_string(),
            border_accent: "#ff9e64".to_string(),
            
            sidebar_bg: "#16161e".to_string(),
            sidebar_header_fg: "#565f89".to_string(),
            sidebar_folder_fg: "#7aa2f7".to_string(),
            sidebar_feed_fg: "#c0caf5".to_string(),
            sidebar_unread_badge_bg: "#24283b".to_string(),
            sidebar_unread_badge_fg: "#7aa2f7".to_string(),
            
            article_list_bg: "#1a1b26".to_string(),
            article_card_bg: "#16161e".to_string(),
            article_card_selected_bg: "#283457".to_string(),
            article_card_selected_border: "#7aa2f7".to_string(),
            article_title_unread: "#c0caf5".to_string(),
            article_title_read: "#787c99".to_string(),
            article_summary_fg: "#9aa5ce".to_string(),
            article_meta_fg: "#7dcfff".to_string(),
            article_unread_dot: "#7aa2f7".to_string(),
            article_star: "#e0af68".to_string(),
            
            reader_bg: "#1a1b26".to_string(),
            reader_header_feed: "#7aa2f7".to_string(),
            reader_header_author: "#9aa5ce".to_string(),
            reader_title: "#c0caf5".to_string(),
            reader_meta: "#565f89".to_string(),
            reader_divider: "#292e42".to_string(),
            reader_body: "#c0caf5".to_string(),
            reader_h1: "#bb9af7".to_string(),
            reader_h2: "#7aa2f7".to_string(),
            reader_h3: "#7dcfff".to_string(),
            reader_quote_border: "#bb9af7".to_string(),
            reader_quote_fg: "#9aa5ce".to_string(),
            reader_code_bg: "#13141c".to_string(),
            reader_code_fg: "#9ece6a".to_string(),
            reader_link: "#7dcfff".to_string(),
            reader_link_url: "#414868".to_string(),
            
            status_bar_bg: "#13141c".to_string(),
            status_bar_fg: "#787c99".to_string(),
            modal_bg: "#16161e".to_string(),
            modal_border: "#7aa2f7".to_string(),
            error_fg: "#f7768e".to_string(),
            success_fg: "#9ece6a".to_string(),
            warning_fg: "#e0af68".to_string(),
        }.into()
    }

    pub fn gruvbox_dark() -> Self {
        ThemeConfig {
            name: "Gruvbox Dark".to_string(),
            description: "Warm retro groove palette".to_string(),
            
            bg: "#282828".to_string(),
            fg: "#ebdbb2".to_string(),
            fg_dim: "#a89984".to_string(),
            fg_subtle: "#665c54".to_string(),
            accent: "#83a598".to_string(),
            accent_subtle: "#3c3836".to_string(),
            selection_bg: "#3c3836".to_string(),
            selection_fg: "#fbf1c7".to_string(),
            
            border_active: "#fabd2f".to_string(),
            border_inactive: "#3c3836".to_string(),
            border_accent: "#fe8019".to_string(),
            
            sidebar_bg: "#1d2021".to_string(),
            sidebar_header_fg: "#928374".to_string(),
            sidebar_folder_fg: "#fabd2f".to_string(),
            sidebar_feed_fg: "#ebdbb2".to_string(),
            sidebar_unread_badge_bg: "#3c3836".to_string(),
            sidebar_unread_badge_fg: "#fabd2f".to_string(),
            
            article_list_bg: "#282828".to_string(),
            article_card_bg: "#1d2021".to_string(),
            article_card_selected_bg: "#3c3836".to_string(),
            article_card_selected_border: "#fabd2f".to_string(),
            article_title_unread: "#fbf1c7".to_string(),
            article_title_read: "#928374".to_string(),
            article_summary_fg: "#bdae93".to_string(),
            article_meta_fg: "#83a598".to_string(),
            article_unread_dot: "#fabd2f".to_string(),
            article_star: "#fe8019".to_string(),
            
            reader_bg: "#282828".to_string(),
            reader_header_feed: "#83a598".to_string(),
            reader_header_author: "#a89984".to_string(),
            reader_title: "#fbf1c7".to_string(),
            reader_meta: "#7c6f64".to_string(),
            reader_divider: "#3c3836".to_string(),
            reader_body: "#ebdbb2".to_string(),
            reader_h1: "#fabd2f".to_string(),
            reader_h2: "#83a598".to_string(),
            reader_h3: "#8ec07c".to_string(),
            reader_quote_border: "#d3869b".to_string(),
            reader_quote_fg: "#a89984".to_string(),
            reader_code_bg: "#1d2021".to_string(),
            reader_code_fg: "#b8bb26".to_string(),
            reader_link: "#83a598".to_string(),
            reader_link_url: "#665c54".to_string(),
            
            status_bar_bg: "#1d2021".to_string(),
            status_bar_fg: "#a89984".to_string(),
            modal_bg: "#282828".to_string(),
            modal_border: "#fabd2f".to_string(),
            error_fg: "#fb4934".to_string(),
            success_fg: "#b8bb26".to_string(),
            warning_fg: "#fabd2f".to_string(),
        }.into()
    }

    pub fn nord() -> Self {
        ThemeConfig {
            name: "Nord".to_string(),
            description: "Arctic, north-bluish clean color palette".to_string(),
            
            bg: "#2e3440".to_string(),
            fg: "#eceff4".to_string(),
            fg_dim: "#d8dee9".to_string(),
            fg_subtle: "#4c566a".to_string(),
            accent: "#88c0d0".to_string(),
            accent_subtle: "#3b4252".to_string(),
            selection_bg: "#3b4252".to_string(),
            selection_fg: "#88c0d0".to_string(),
            
            border_active: "#88c0d0".to_string(),
            border_inactive: "#3b4252".to_string(),
            border_accent: "#81a1c1".to_string(),
            
            sidebar_bg: "#242933".to_string(),
            sidebar_header_fg: "#4c566a".to_string(),
            sidebar_folder_fg: "#81a1c1".to_string(),
            sidebar_feed_fg: "#e5e9f0".to_string(),
            sidebar_unread_badge_bg: "#3b4252".to_string(),
            sidebar_unread_badge_fg: "#88c0d0".to_string(),
            
            article_list_bg: "#2e3440".to_string(),
            article_card_bg: "#242933".to_string(),
            article_card_selected_bg: "#3b4252".to_string(),
            article_card_selected_border: "#88c0d0".to_string(),
            article_title_unread: "#eceff4".to_string(),
            article_title_read: "#7e889b".to_string(),
            article_summary_fg: "#d8dee9".to_string(),
            article_meta_fg: "#81a1c1".to_string(),
            article_unread_dot: "#88c0d0".to_string(),
            article_star: "#ebcb8b".to_string(),
            
            reader_bg: "#2e3440".to_string(),
            reader_header_feed: "#81a1c1".to_string(),
            reader_header_author: "#d8dee9".to_string(),
            reader_title: "#eceff4".to_string(),
            reader_meta: "#4c566a".to_string(),
            reader_divider: "#3b4252".to_string(),
            reader_body: "#e5e9f0".to_string(),
            reader_h1: "#88c0d0".to_string(),
            reader_h2: "#81a1c1".to_string(),
            reader_h3: "#5e81ac".to_string(),
            reader_quote_border: "#b48ead".to_string(),
            reader_quote_fg: "#d8dee9".to_string(),
            reader_code_bg: "#242933".to_string(),
            reader_code_fg: "#a3be8c".to_string(),
            reader_link: "#88c0d0".to_string(),
            reader_link_url: "#4c566a".to_string(),
            
            status_bar_bg: "#242933".to_string(),
            status_bar_fg: "#d8dee9".to_string(),
            modal_bg: "#2e3440".to_string(),
            modal_border: "#88c0d0".to_string(),
            error_fg: "#bf616a".to_string(),
            success_fg: "#a3be8c".to_string(),
            warning_fg: "#ebcb8b".to_string(),
        }.into()
    }

    pub fn dracula() -> Self {
        ThemeConfig {
            name: "Dracula".to_string(),
            description: "Dark theme for vampires".to_string(),
            
            bg: "#282a36".to_string(),
            fg: "#f8f8f2".to_string(),
            fg_dim: "#bfbfbf".to_string(),
            fg_subtle: "#6272a4".to_string(),
            accent: "#bd93f9".to_string(),
            accent_subtle: "#44475a".to_string(),
            selection_bg: "#44475a".to_string(),
            selection_fg: "#f8f8f2".to_string(),
            
            border_active: "#bd93f9".to_string(),
            border_inactive: "#44475a".to_string(),
            border_accent: "#ff79c6".to_string(),
            
            sidebar_bg: "#21222c".to_string(),
            sidebar_header_fg: "#6272a4".to_string(),
            sidebar_folder_fg: "#bd93f9".to_string(),
            sidebar_feed_fg: "#f8f8f2".to_string(),
            sidebar_unread_badge_bg: "#44475a".to_string(),
            sidebar_unread_badge_fg: "#ff79c6".to_string(),
            
            article_list_bg: "#282a36".to_string(),
            article_card_bg: "#21222c".to_string(),
            article_card_selected_bg: "#44475a".to_string(),
            article_card_selected_border: "#bd93f9".to_string(),
            article_title_unread: "#f8f8f2".to_string(),
            article_title_read: "#787c99".to_string(),
            article_summary_fg: "#d6acff".to_string(),
            article_meta_fg: "#8be9fd".to_string(),
            article_unread_dot: "#ff79c6".to_string(),
            article_star: "#f1fa8c".to_string(),
            
            reader_bg: "#282a36".to_string(),
            reader_header_feed: "#bd93f9".to_string(),
            reader_header_author: "#f8f8f2".to_string(),
            reader_title: "#f8f8f2".to_string(),
            reader_meta: "#6272a4".to_string(),
            reader_divider: "#44475a".to_string(),
            reader_body: "#f8f8f2".to_string(),
            reader_h1: "#ff79c6".to_string(),
            reader_h2: "#bd93f9".to_string(),
            reader_h3: "#8be9fd".to_string(),
            reader_quote_border: "#50fa7b".to_string(),
            reader_quote_fg: "#f1fa8c".to_string(),
            reader_code_bg: "#1e1f29".to_string(),
            reader_code_fg: "#50fa7b".to_string(),
            reader_link: "#8be9fd".to_string(),
            reader_link_url: "#6272a4".to_string(),
            
            status_bar_bg: "#1e1f29".to_string(),
            status_bar_fg: "#6272a4".to_string(),
            modal_bg: "#282a36".to_string(),
            modal_border: "#bd93f9".to_string(),
            error_fg: "#ff5555".to_string(),
            success_fg: "#50fa7b".to_string(),
            warning_fg: "#ffb86c".to_string(),
        }.into()
    }

    pub fn minimal_mono() -> Self {
        ThemeConfig {
            name: "Minimal Monochrome".to_string(),
            description: "High contrast minimalist black and white terminal".to_string(),
            
            bg: "#000000".to_string(),
            fg: "#ffffff".to_string(),
            fg_dim: "#888888".to_string(),
            fg_subtle: "#444444".to_string(),
            accent: "#ffffff".to_string(),
            accent_subtle: "#222222".to_string(),
            selection_bg: "#ffffff".to_string(),
            selection_fg: "#000000".to_string(),
            
            border_active: "#ffffff".to_string(),
            border_inactive: "#333333".to_string(),
            border_accent: "#aaaaaa".to_string(),
            
            sidebar_bg: "#000000".to_string(),
            sidebar_header_fg: "#666666".to_string(),
            sidebar_folder_fg: "#ffffff".to_string(),
            sidebar_feed_fg: "#cccccc".to_string(),
            sidebar_unread_badge_bg: "#222222".to_string(),
            sidebar_unread_badge_fg: "#ffffff".to_string(),
            
            article_list_bg: "#000000".to_string(),
            article_card_bg: "#0d0d0d".to_string(),
            article_card_selected_bg: "#222222".to_string(),
            article_card_selected_border: "#ffffff".to_string(),
            article_title_unread: "#ffffff".to_string(),
            article_title_read: "#777777".to_string(),
            article_summary_fg: "#999999".to_string(),
            article_meta_fg: "#ffffff".to_string(),
            article_unread_dot: "#ffffff".to_string(),
            article_star: "#ffffff".to_string(),
            
            reader_bg: "#000000".to_string(),
            reader_header_feed: "#ffffff".to_string(),
            reader_header_author: "#888888".to_string(),
            reader_title: "#ffffff".to_string(),
            reader_meta: "#555555".to_string(),
            reader_divider: "#333333".to_string(),
            reader_body: "#dddddd".to_string(),
            reader_h1: "#ffffff".to_string(),
            reader_h2: "#eeeeee".to_string(),
            reader_h3: "#cccccc".to_string(),
            reader_quote_border: "#888888".to_string(),
            reader_quote_fg: "#aaaaaa".to_string(),
            reader_code_bg: "#111111".to_string(),
            reader_code_fg: "#ffffff".to_string(),
            reader_link: "#ffffff".to_string(),
            reader_link_url: "#555555".to_string(),
            
            status_bar_bg: "#111111".to_string(),
            status_bar_fg: "#888888".to_string(),
            modal_bg: "#000000".to_string(),
            modal_border: "#ffffff".to_string(),
            error_fg: "#ffffff".to_string(),
            success_fg: "#ffffff".to_string(),
            warning_fg: "#ffffff".to_string(),
        }.into()
    }

    pub fn all_presets() -> Vec<Theme> {
        vec![
            Self::netnewswire_dark(),
            Self::netnewswire_light(),
            Self::catppuccin_mocha(),
            Self::tokyo_night(),
            Self::gruvbox_dark(),
            Self::nord(),
            Self::dracula(),
            Self::minimal_mono(),
        ]
    }

    pub fn by_name(name: &str) -> Self {
        for theme in Self::all_presets() {
            if theme.config.name.eq_ignore_ascii_case(name) {
                return theme;
            }
        }
        Self::netnewswire_dark()
    }

    // Helper styles
    pub fn title_style(&self, unread: bool) -> Style {
        if unread {
            Style::default().fg(self.article_title_unread).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.article_title_read)
        }
    }

    pub fn active_border_style(&self) -> Style {
        Style::default().fg(self.border_active).add_modifier(Modifier::BOLD)
    }

    pub fn inactive_border_style(&self) -> Style {
        Style::default().fg(self.border_inactive)
    }
}
