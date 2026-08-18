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


/// A compact palette from which a full [`ThemeConfig`] is derived.
///
/// The full config names about fifty roles, but a palette only really has a
/// dozen decisions in it. Writing new themes as palettes keeps the derived
/// roles consistent across themes — and keeps a new theme to a dozen lines
/// instead of fifty near-identical ones.
pub struct Palette {
    pub name: &'static str,
    pub description: &'static str,
    /// Main background.
    pub bg: &'static str,
    /// Recessed background: sidebar, cards, status bar.
    pub bg_alt: &'static str,
    /// Selected-row background.
    pub bg_sel: &'static str,
    pub fg: &'static str,
    pub fg_dim: &'static str,
    pub fg_subtle: &'static str,
    pub accent: &'static str,
    pub red: &'static str,
    pub green: &'static str,
    pub yellow: &'static str,
    pub blue: &'static str,
    pub magenta: &'static str,
    pub cyan: &'static str,
}

impl From<Palette> for Theme {
    fn from(p: Palette) -> Self {
        ThemeConfig::from(p).into()
    }
}

impl From<Palette> for ThemeConfig {
    fn from(p: Palette) -> Self {
        ThemeConfig {
            name: p.name.to_string(),
            description: p.description.to_string(),

            bg: p.bg.to_string(),
            fg: p.fg.to_string(),
            fg_dim: p.fg_dim.to_string(),
            fg_subtle: p.fg_subtle.to_string(),
            accent: p.accent.to_string(),
            accent_subtle: p.bg_sel.to_string(),
            selection_bg: p.bg_sel.to_string(),
            selection_fg: p.accent.to_string(),

            border_active: p.accent.to_string(),
            border_inactive: p.bg_sel.to_string(),
            border_accent: p.blue.to_string(),

            sidebar_bg: p.bg_alt.to_string(),
            sidebar_header_fg: p.fg_subtle.to_string(),
            sidebar_folder_fg: p.blue.to_string(),
            sidebar_feed_fg: p.fg.to_string(),
            sidebar_unread_badge_bg: p.bg_sel.to_string(),
            sidebar_unread_badge_fg: p.accent.to_string(),

            article_list_bg: p.bg.to_string(),
            article_card_bg: p.bg_alt.to_string(),
            article_card_selected_bg: p.bg_sel.to_string(),
            article_card_selected_border: p.accent.to_string(),
            article_title_unread: p.fg.to_string(),
            article_title_read: p.fg_subtle.to_string(),
            article_summary_fg: p.fg_dim.to_string(),
            article_meta_fg: p.cyan.to_string(),
            article_unread_dot: p.accent.to_string(),
            article_star: p.yellow.to_string(),

            reader_bg: p.bg.to_string(),
            reader_header_feed: p.blue.to_string(),
            reader_header_author: p.fg_dim.to_string(),
            reader_title: p.fg.to_string(),
            reader_meta: p.fg_subtle.to_string(),
            reader_divider: p.bg_sel.to_string(),
            reader_body: p.fg.to_string(),
            reader_h1: p.accent.to_string(),
            reader_h2: p.blue.to_string(),
            reader_h3: p.cyan.to_string(),
            reader_quote_border: p.magenta.to_string(),
            reader_quote_fg: p.fg_dim.to_string(),
            reader_code_bg: p.bg_alt.to_string(),
            reader_code_fg: p.green.to_string(),
            reader_link: p.accent.to_string(),
            reader_link_url: p.cyan.to_string(),

            status_bar_bg: p.bg_alt.to_string(),
            status_bar_fg: p.fg_dim.to_string(),
            modal_bg: p.bg.to_string(),
            modal_border: p.accent.to_string(),
            error_fg: p.red.to_string(),
            success_fg: p.green.to_string(),
            warning_fg: p.yellow.to_string(),
        }
    }
}

pub fn parse_color(color_str: &str) -> Color {
    let s = color_str.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("reset") {
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
    pub fn ratarss_dark() -> Self {
        ThemeConfig {
            name: "RataRSS Dark".to_string(),
            description: "Default sleek dark RSS desktop theme with crisp blue accents".to_string(),

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
        }
        .into()
    }

    #[allow(dead_code)]
    pub fn netnewswire_dark() -> Self {
        Self::ratarss_dark()
    }

    pub fn ratarss_light() -> Self {
        ThemeConfig {
            name: "RataRSS Light".to_string(),
            description: "Clean Apple macOS inspired crisp light theme".to_string(),

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
        }
        .into()
    }

    #[allow(dead_code)]
    pub fn netnewswire_light() -> Self {
        Self::ratarss_light()
    }

    pub fn catppuccin_mocha() -> Self {
        ThemeConfig {
            name: "Catppuccin Mocha".to_string(),
            description: "Soothing pastel dark theme with lavender and sky blue".to_string(),

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
        }
        .into()
    }

    pub fn catppuccin_macchiato() -> Self {
        ThemeConfig {
            name: "Catppuccin Macchiato".to_string(),
            description: "Medium dark comforting Catppuccin palette".to_string(),

            bg: "#24273a".to_string(),
            fg: "#cad3f5".to_string(),
            fg_dim: "#a5adcb".to_string(),
            fg_subtle: "#5b6078".to_string(),
            accent: "#8aadf4".to_string(),
            accent_subtle: "#363a4f".to_string(),
            selection_bg: "#363a4f".to_string(),
            selection_fg: "#cad3f5".to_string(),

            border_active: "#8aadf4".to_string(),
            border_inactive: "#363a4f".to_string(),
            border_accent: "#f5a97f".to_string(),

            sidebar_bg: "#1e2030".to_string(),
            sidebar_header_fg: "#6e738d".to_string(),
            sidebar_folder_fg: "#8aadf4".to_string(),
            sidebar_feed_fg: "#b8c0e0".to_string(),
            sidebar_unread_badge_bg: "#363a4f".to_string(),
            sidebar_unread_badge_fg: "#8aadf4".to_string(),

            article_list_bg: "#24273a".to_string(),
            article_card_bg: "#1e2030".to_string(),
            article_card_selected_bg: "#363a4f".to_string(),
            article_card_selected_border: "#8aadf4".to_string(),
            article_title_unread: "#cad3f5".to_string(),
            article_title_read: "#8087a2".to_string(),
            article_summary_fg: "#a5adcb".to_string(),
            article_meta_fg: "#8aadf4".to_string(),
            article_unread_dot: "#8aadf4".to_string(),
            article_star: "#eed49f".to_string(),

            reader_bg: "#24273a".to_string(),
            reader_header_feed: "#8aadf4".to_string(),
            reader_header_author: "#a5adcb".to_string(),
            reader_title: "#cad3f5".to_string(),
            reader_meta: "#6e738d".to_string(),
            reader_divider: "#363a4f".to_string(),
            reader_body: "#cad3f5".to_string(),
            reader_h1: "#c6a0f6".to_string(),
            reader_h2: "#8aadf4".to_string(),
            reader_h3: "#7dc4e4".to_string(),
            reader_quote_border: "#f5bde6".to_string(),
            reader_quote_fg: "#a5adcb".to_string(),
            reader_code_bg: "#181926".to_string(),
            reader_code_fg: "#a6da95".to_string(),
            reader_link: "#91d7e3".to_string(),
            reader_link_url: "#5b6078".to_string(),

            status_bar_bg: "#181926".to_string(),
            status_bar_fg: "#a5adcb".to_string(),
            modal_bg: "#1e2030".to_string(),
            modal_border: "#8aadf4".to_string(),
            error_fg: "#ed8796".to_string(),
            success_fg: "#a6da95".to_string(),
            warning_fg: "#eed49f".to_string(),
        }
        .into()
    }

    pub fn catppuccin_frappe() -> Self {
        ThemeConfig {
            name: "Catppuccin Frappé".to_string(),
            description: "Subdued, low-contrast dark Catppuccin palette".to_string(),

            bg: "#303446".to_string(),
            fg: "#c6d0f5".to_string(),
            fg_dim: "#a5b4fc".to_string(),
            fg_subtle: "#626880".to_string(),
            accent: "#8caaee".to_string(),
            accent_subtle: "#414559".to_string(),
            selection_bg: "#414559".to_string(),
            selection_fg: "#c6d0f5".to_string(),

            border_active: "#8caaee".to_string(),
            border_inactive: "#414559".to_string(),
            border_accent: "#ef9f76".to_string(),

            sidebar_bg: "#292c3c".to_string(),
            sidebar_header_fg: "#737994".to_string(),
            sidebar_folder_fg: "#8caaee".to_string(),
            sidebar_feed_fg: "#b5bfe2".to_string(),
            sidebar_unread_badge_bg: "#414559".to_string(),
            sidebar_unread_badge_fg: "#8caaee".to_string(),

            article_list_bg: "#303446".to_string(),
            article_card_bg: "#292c3c".to_string(),
            article_card_selected_bg: "#414559".to_string(),
            article_card_selected_border: "#8caaee".to_string(),
            article_title_unread: "#c6d0f5".to_string(),
            article_title_read: "#838ba7".to_string(),
            article_summary_fg: "#a5adce".to_string(),
            article_meta_fg: "#8caaee".to_string(),
            article_unread_dot: "#8caaee".to_string(),
            article_star: "#e5c890".to_string(),

            reader_bg: "#303446".to_string(),
            reader_header_feed: "#8caaee".to_string(),
            reader_header_author: "#a5adce".to_string(),
            reader_title: "#c6d0f5".to_string(),
            reader_meta: "#737994".to_string(),
            reader_divider: "#414559".to_string(),
            reader_body: "#c6d0f5".to_string(),
            reader_h1: "#ca9ee6".to_string(),
            reader_h2: "#8caaee".to_string(),
            reader_h3: "#85c1dc".to_string(),
            reader_quote_border: "#f4b8e4".to_string(),
            reader_quote_fg: "#a5adce".to_string(),
            reader_code_bg: "#232634".to_string(),
            reader_code_fg: "#a6d189".to_string(),
            reader_link: "#81c8be".to_string(),
            reader_link_url: "#626880".to_string(),

            status_bar_bg: "#232634".to_string(),
            status_bar_fg: "#a5adce".to_string(),
            modal_bg: "#292c3c".to_string(),
            modal_border: "#8caaee".to_string(),
            error_fg: "#e78284".to_string(),
            success_fg: "#a6d189".to_string(),
            warning_fg: "#e5c890".to_string(),
        }
        .into()
    }

    pub fn catppuccin_latte() -> Self {
        ThemeConfig {
            name: "Catppuccin Latte".to_string(),
            description: "Warm, soothing light pastel palette".to_string(),

            bg: "#eff1f5".to_string(),
            fg: "#4c4f69".to_string(),
            fg_dim: "#6c6f85".to_string(),
            fg_subtle: "#9ca0b0".to_string(),
            accent: "#1e66f5".to_string(),
            accent_subtle: "#ccd0da".to_string(),
            selection_bg: "#dce0e8".to_string(),
            selection_fg: "#1e66f5".to_string(),

            border_active: "#1e66f5".to_string(),
            border_inactive: "#ccd0da".to_string(),
            border_accent: "#fe640b".to_string(),

            sidebar_bg: "#e6e9ef".to_string(),
            sidebar_header_fg: "#7c7f93".to_string(),
            sidebar_folder_fg: "#1e66f5".to_string(),
            sidebar_feed_fg: "#4c4f69".to_string(),
            sidebar_unread_badge_bg: "#ccd0da".to_string(),
            sidebar_unread_badge_fg: "#1e66f5".to_string(),

            article_list_bg: "#eff1f5".to_string(),
            article_card_bg: "#e6e9ef".to_string(),
            article_card_selected_bg: "#dce0e8".to_string(),
            article_card_selected_border: "#1e66f5".to_string(),
            article_title_unread: "#4c4f69".to_string(),
            article_title_read: "#8c8fa1".to_string(),
            article_summary_fg: "#6c6f85".to_string(),
            article_meta_fg: "#1e66f5".to_string(),
            article_unread_dot: "#1e66f5".to_string(),
            article_star: "#df8e1d".to_string(),

            reader_bg: "#eff1f5".to_string(),
            reader_header_feed: "#1e66f5".to_string(),
            reader_header_author: "#6c6f85".to_string(),
            reader_title: "#4c4f69".to_string(),
            reader_meta: "#7c7f93".to_string(),
            reader_divider: "#ccd0da".to_string(),
            reader_body: "#4c4f69".to_string(),
            reader_h1: "#8839ef".to_string(),
            reader_h2: "#1e66f5".to_string(),
            reader_h3: "#209fb5".to_string(),
            reader_quote_border: "#ea76cb".to_string(),
            reader_quote_fg: "#6c6f85".to_string(),
            reader_code_bg: "#e6e9ef".to_string(),
            reader_code_fg: "#40a02b".to_string(),
            reader_link: "#1e66f5".to_string(),
            reader_link_url: "#7c7f93".to_string(),

            status_bar_bg: "#dce0e8".to_string(),
            status_bar_fg: "#6c6f85".to_string(),
            modal_bg: "#eff1f5".to_string(),
            modal_border: "#1e66f5".to_string(),
            error_fg: "#d20f39".to_string(),
            success_fg: "#40a02b".to_string(),
            warning_fg: "#df8e1d".to_string(),
        }
        .into()
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
        }
        .into()
    }

    pub fn tokyo_night_storm() -> Self {
        ThemeConfig {
            name: "Tokyo Night Storm".to_string(),
            description: "Slightly brighter Tokyo Night Storm variant".to_string(),

            bg: "#24283b".to_string(),
            fg: "#c0caf5".to_string(),
            fg_dim: "#9aa5ce".to_string(),
            fg_subtle: "#565f89".to_string(),
            accent: "#7aa2f7".to_string(),
            accent_subtle: "#2f354f".to_string(),
            selection_bg: "#2f354f".to_string(),
            selection_fg: "#c0caf5".to_string(),

            border_active: "#7aa2f7".to_string(),
            border_inactive: "#2f354f".to_string(),
            border_accent: "#ff9e64".to_string(),

            sidebar_bg: "#1f2335".to_string(),
            sidebar_header_fg: "#565f89".to_string(),
            sidebar_folder_fg: "#7aa2f7".to_string(),
            sidebar_feed_fg: "#c0caf5".to_string(),
            sidebar_unread_badge_bg: "#2f354f".to_string(),
            sidebar_unread_badge_fg: "#7aa2f7".to_string(),

            article_list_bg: "#24283b".to_string(),
            article_card_bg: "#1f2335".to_string(),
            article_card_selected_bg: "#2f354f".to_string(),
            article_card_selected_border: "#7aa2f7".to_string(),
            article_title_unread: "#c0caf5".to_string(),
            article_title_read: "#787c99".to_string(),
            article_summary_fg: "#9aa5ce".to_string(),
            article_meta_fg: "#7dcfff".to_string(),
            article_unread_dot: "#7aa2f7".to_string(),
            article_star: "#e0af68".to_string(),

            reader_bg: "#24283b".to_string(),
            reader_header_feed: "#7aa2f7".to_string(),
            reader_header_author: "#9aa5ce".to_string(),
            reader_title: "#c0caf5".to_string(),
            reader_meta: "#565f89".to_string(),
            reader_divider: "#2f354f".to_string(),
            reader_body: "#c0caf5".to_string(),
            reader_h1: "#bb9af7".to_string(),
            reader_h2: "#7aa2f7".to_string(),
            reader_h3: "#7dcfff".to_string(),
            reader_quote_border: "#bb9af7".to_string(),
            reader_quote_fg: "#9aa5ce".to_string(),
            reader_code_bg: "#1f2335".to_string(),
            reader_code_fg: "#9ece6a".to_string(),
            reader_link: "#7dcfff".to_string(),
            reader_link_url: "#565f89".to_string(),

            status_bar_bg: "#1f2335".to_string(),
            status_bar_fg: "#9aa5ce".to_string(),
            modal_bg: "#1f2335".to_string(),
            modal_border: "#7aa2f7".to_string(),
            error_fg: "#f7768e".to_string(),
            success_fg: "#9ece6a".to_string(),
            warning_fg: "#e0af68".to_string(),
        }
        .into()
    }

    pub fn gruvbox_dark() -> Self {
        ThemeConfig {
            name: "Gruvbox Dark".to_string(),
            description: "Warm retro groove palette with rich ochre and amber".to_string(),

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
        }
        .into()
    }

    pub fn gruvbox_light() -> Self {
        ThemeConfig {
            name: "Gruvbox Light".to_string(),
            description: "Warm retro groove palette on warm parchment".to_string(),

            bg: "#fbf1c7".to_string(),
            fg: "#3c3836".to_string(),
            fg_dim: "#7c6f64".to_string(),
            fg_subtle: "#bdae93".to_string(),
            accent: "#076678".to_string(),
            accent_subtle: "#ebdbb2".to_string(),
            selection_bg: "#ebdbb2".to_string(),
            selection_fg: "#282828".to_string(),

            border_active: "#b57614".to_string(),
            border_inactive: "#d5c4a1".to_string(),
            border_accent: "#af3a03".to_string(),

            sidebar_bg: "#f2e5bc".to_string(),
            sidebar_header_fg: "#7c6f64".to_string(),
            sidebar_folder_fg: "#b57614".to_string(),
            sidebar_feed_fg: "#3c3836".to_string(),
            sidebar_unread_badge_bg: "#ebdbb2".to_string(),
            sidebar_unread_badge_fg: "#b57614".to_string(),

            article_list_bg: "#fbf1c7".to_string(),
            article_card_bg: "#f2e5bc".to_string(),
            article_card_selected_bg: "#ebdbb2".to_string(),
            article_card_selected_border: "#b57614".to_string(),
            article_title_unread: "#282828".to_string(),
            article_title_read: "#7c6f64".to_string(),
            article_summary_fg: "#504945".to_string(),
            article_meta_fg: "#076678".to_string(),
            article_unread_dot: "#b57614".to_string(),
            article_star: "#af3a03".to_string(),

            reader_bg: "#fbf1c7".to_string(),
            reader_header_feed: "#076678".to_string(),
            reader_header_author: "#7c6f64".to_string(),
            reader_title: "#282828".to_string(),
            reader_meta: "#928374".to_string(),
            reader_divider: "#d5c4a1".to_string(),
            reader_body: "#3c3836".to_string(),
            reader_h1: "#b57614".to_string(),
            reader_h2: "#076678".to_string(),
            reader_h3: "#427b58".to_string(),
            reader_quote_border: "#8f3f71".to_string(),
            reader_quote_fg: "#7c6f64".to_string(),
            reader_code_bg: "#f2e5bc".to_string(),
            reader_code_fg: "#79740e".to_string(),
            reader_link: "#076678".to_string(),
            reader_link_url: "#928374".to_string(),

            status_bar_bg: "#ebdbb2".to_string(),
            status_bar_fg: "#504945".to_string(),
            modal_bg: "#fbf1c7".to_string(),
            modal_border: "#b57614".to_string(),
            error_fg: "#9d0006".to_string(),
            success_fg: "#79740e".to_string(),
            warning_fg: "#b57614".to_string(),
        }
        .into()
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
        }
        .into()
    }

    pub fn dracula() -> Self {
        ThemeConfig {
            name: "Dracula".to_string(),
            description: "Classic gothic dark theme with purple and pink accents".to_string(),

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
        }
        .into()
    }

    pub fn solarized_dark() -> Self {
        ThemeConfig {
            name: "Solarized Dark".to_string(),
            description: "Precision dark palette by Ethan Schoonover".to_string(),

            bg: "#002b36".to_string(),
            fg: "#839496".to_string(),
            fg_dim: "#657b83".to_string(),
            fg_subtle: "#586e75".to_string(),
            accent: "#268bd2".to_string(),
            accent_subtle: "#073642".to_string(),
            selection_bg: "#073642".to_string(),
            selection_fg: "#93a1a1".to_string(),

            border_active: "#268bd2".to_string(),
            border_inactive: "#073642".to_string(),
            border_accent: "#cb4b16".to_string(),

            sidebar_bg: "#00212b".to_string(),
            sidebar_header_fg: "#586e75".to_string(),
            sidebar_folder_fg: "#268bd2".to_string(),
            sidebar_feed_fg: "#93a1a1".to_string(),
            sidebar_unread_badge_bg: "#073642".to_string(),
            sidebar_unread_badge_fg: "#2aa198".to_string(),

            article_list_bg: "#002b36".to_string(),
            article_card_bg: "#00212b".to_string(),
            article_card_selected_bg: "#073642".to_string(),
            article_card_selected_border: "#268bd2".to_string(),
            article_title_unread: "#fdf6e3".to_string(),
            article_title_read: "#657b83".to_string(),
            article_summary_fg: "#839496".to_string(),
            article_meta_fg: "#2aa198".to_string(),
            article_unread_dot: "#268bd2".to_string(),
            article_star: "#b58900".to_string(),

            reader_bg: "#002b36".to_string(),
            reader_header_feed: "#268bd2".to_string(),
            reader_header_author: "#839496".to_string(),
            reader_title: "#eee8d5".to_string(),
            reader_meta: "#586e75".to_string(),
            reader_divider: "#073642".to_string(),
            reader_body: "#93a1a1".to_string(),
            reader_h1: "#268bd2".to_string(),
            reader_h2: "#2aa198".to_string(),
            reader_h3: "#859900".to_string(),
            reader_quote_border: "#6c71c4".to_string(),
            reader_quote_fg: "#839496".to_string(),
            reader_code_bg: "#00212b".to_string(),
            reader_code_fg: "#859900".to_string(),
            reader_link: "#2aa198".to_string(),
            reader_link_url: "#586e75".to_string(),

            status_bar_bg: "#00212b".to_string(),
            status_bar_fg: "#657b83".to_string(),
            modal_bg: "#002b36".to_string(),
            modal_border: "#268bd2".to_string(),
            error_fg: "#dc322f".to_string(),
            success_fg: "#859900".to_string(),
            warning_fg: "#b58900".to_string(),
        }
        .into()
    }

    pub fn solarized_light() -> Self {
        ThemeConfig {
            name: "Solarized Light".to_string(),
            description: "Warm, light Solarized palette".to_string(),

            bg: "#fdf6e3".to_string(),
            fg: "#657b83".to_string(),
            fg_dim: "#839496".to_string(),
            fg_subtle: "#93a1a1".to_string(),
            accent: "#268bd2".to_string(),
            accent_subtle: "#eee8d5".to_string(),
            selection_bg: "#eee8d5".to_string(),
            selection_fg: "#073642".to_string(),

            border_active: "#268bd2".to_string(),
            border_inactive: "#eee8d5".to_string(),
            border_accent: "#cb4b16".to_string(),

            sidebar_bg: "#f5efdc".to_string(),
            sidebar_header_fg: "#93a1a1".to_string(),
            sidebar_folder_fg: "#268bd2".to_string(),
            sidebar_feed_fg: "#586e75".to_string(),
            sidebar_unread_badge_bg: "#eee8d5".to_string(),
            sidebar_unread_badge_fg: "#2aa198".to_string(),

            article_list_bg: "#fdf6e3".to_string(),
            article_card_bg: "#f5efdc".to_string(),
            article_card_selected_bg: "#eee8d5".to_string(),
            article_card_selected_border: "#268bd2".to_string(),
            article_title_unread: "#073642".to_string(),
            article_title_read: "#93a1a1".to_string(),
            article_summary_fg: "#657b83".to_string(),
            article_meta_fg: "#2aa198".to_string(),
            article_unread_dot: "#268bd2".to_string(),
            article_star: "#b58900".to_string(),

            reader_bg: "#fdf6e3".to_string(),
            reader_header_feed: "#268bd2".to_string(),
            reader_header_author: "#657b83".to_string(),
            reader_title: "#073642".to_string(),
            reader_meta: "#93a1a1".to_string(),
            reader_divider: "#eee8d5".to_string(),
            reader_body: "#586e75".to_string(),
            reader_h1: "#268bd2".to_string(),
            reader_h2: "#2aa198".to_string(),
            reader_h3: "#859900".to_string(),
            reader_quote_border: "#6c71c4".to_string(),
            reader_quote_fg: "#657b83".to_string(),
            reader_code_bg: "#eee8d5".to_string(),
            reader_code_fg: "#859900".to_string(),
            reader_link: "#2aa198".to_string(),
            reader_link_url: "#93a1a1".to_string(),

            status_bar_bg: "#eee8d5".to_string(),
            status_bar_fg: "#657b83".to_string(),
            modal_bg: "#fdf6e3".to_string(),
            modal_border: "#268bd2".to_string(),
            error_fg: "#dc322f".to_string(),
            success_fg: "#859900".to_string(),
            warning_fg: "#b58900".to_string(),
        }
        .into()
    }

    pub fn rose_pine() -> Self {
        ThemeConfig {
            name: "Rosé Pine".to_string(),
            description: "All natural pine, faux fur and warmth for cozy reading".to_string(),

            bg: "#191724".to_string(),
            fg: "#e0def4".to_string(),
            fg_dim: "#908caa".to_string(),
            fg_subtle: "#6e6a86".to_string(),
            accent: "#ebbcba".to_string(),
            accent_subtle: "#26233a".to_string(),
            selection_bg: "#26233a".to_string(),
            selection_fg: "#e0def4".to_string(),

            border_active: "#ebbcba".to_string(),
            border_inactive: "#26233a".to_string(),
            border_accent: "#f6c177".to_string(),

            sidebar_bg: "#1f1d2e".to_string(),
            sidebar_header_fg: "#6e6a86".to_string(),
            sidebar_folder_fg: "#ebbcba".to_string(),
            sidebar_feed_fg: "#e0def4".to_string(),
            sidebar_unread_badge_bg: "#26233a".to_string(),
            sidebar_unread_badge_fg: "#eb6f92".to_string(),

            article_list_bg: "#191724".to_string(),
            article_card_bg: "#1f1d2e".to_string(),
            article_card_selected_bg: "#26233a".to_string(),
            article_card_selected_border: "#ebbcba".to_string(),
            article_title_unread: "#e0def4".to_string(),
            article_title_read: "#908caa".to_string(),
            article_summary_fg: "#908caa".to_string(),
            article_meta_fg: "#9ccfd8".to_string(),
            article_unread_dot: "#eb6f92".to_string(),
            article_star: "#f6c177".to_string(),

            reader_bg: "#191724".to_string(),
            reader_header_feed: "#ebbcba".to_string(),
            reader_header_author: "#908caa".to_string(),
            reader_title: "#e0def4".to_string(),
            reader_meta: "#6e6a86".to_string(),
            reader_divider: "#26233a".to_string(),
            reader_body: "#e0def4".to_string(),
            reader_h1: "#c4a7e7".to_string(),
            reader_h2: "#ebbcba".to_string(),
            reader_h3: "#9ccfd8".to_string(),
            reader_quote_border: "#eb6f92".to_string(),
            reader_quote_fg: "#908caa".to_string(),
            reader_code_bg: "#1f1d2e".to_string(),
            reader_code_fg: "#31748f".to_string(),
            reader_link: "#9ccfd8".to_string(),
            reader_link_url: "#6e6a86".to_string(),

            status_bar_bg: "#1f1d2e".to_string(),
            status_bar_fg: "#908caa".to_string(),
            modal_bg: "#191724".to_string(),
            modal_border: "#ebbcba".to_string(),
            error_fg: "#eb6f92".to_string(),
            success_fg: "#9ccfd8".to_string(),
            warning_fg: "#f6c177".to_string(),
        }
        .into()
    }

    pub fn rose_pine_dawn() -> Self {
        ThemeConfig {
            name: "Rosé Pine Dawn".to_string(),
            description: "Warm light pastel dawn palette".to_string(),

            bg: "#faf4ed".to_string(),
            fg: "#575279".to_string(),
            fg_dim: "#797593".to_string(),
            fg_subtle: "#9893a5".to_string(),
            accent: "#d7827e".to_string(),
            accent_subtle: "#f2e9de".to_string(),
            selection_bg: "#f2e9de".to_string(),
            selection_fg: "#575279".to_string(),

            border_active: "#d7827e".to_string(),
            border_inactive: "#f2e9de".to_string(),
            border_accent: "#ea9d34".to_string(),

            sidebar_bg: "#f2e9de".to_string(),
            sidebar_header_fg: "#9893a5".to_string(),
            sidebar_folder_fg: "#d7827e".to_string(),
            sidebar_feed_fg: "#575279".to_string(),
            sidebar_unread_badge_bg: "#faf4ed".to_string(),
            sidebar_unread_badge_fg: "#b4637a".to_string(),

            article_list_bg: "#faf4ed".to_string(),
            article_card_bg: "#f2e9de".to_string(),
            article_card_selected_bg: "#fffaf3".to_string(),
            article_card_selected_border: "#d7827e".to_string(),
            article_title_unread: "#575279".to_string(),
            article_title_read: "#9893a5".to_string(),
            article_summary_fg: "#797593".to_string(),
            article_meta_fg: "#56949f".to_string(),
            article_unread_dot: "#b4637a".to_string(),
            article_star: "#ea9d34".to_string(),

            reader_bg: "#faf4ed".to_string(),
            reader_header_feed: "#d7827e".to_string(),
            reader_header_author: "#797593".to_string(),
            reader_title: "#575279".to_string(),
            reader_meta: "#9893a5".to_string(),
            reader_divider: "#f2e9de".to_string(),
            reader_body: "#575279".to_string(),
            reader_h1: "#907aa9".to_string(),
            reader_h2: "#d7827e".to_string(),
            reader_h3: "#56949f".to_string(),
            reader_quote_border: "#b4637a".to_string(),
            reader_quote_fg: "#797593".to_string(),
            reader_code_bg: "#f2e9de".to_string(),
            reader_code_fg: "#286983".to_string(),
            reader_link: "#56949f".to_string(),
            reader_link_url: "#9893a5".to_string(),

            status_bar_bg: "#f2e9de".to_string(),
            status_bar_fg: "#797593".to_string(),
            modal_bg: "#faf4ed".to_string(),
            modal_border: "#d7827e".to_string(),
            error_fg: "#b4637a".to_string(),
            success_fg: "#56949f".to_string(),
            warning_fg: "#ea9d34".to_string(),
        }
        .into()
    }

    pub fn rose_pine_moon() -> Self {
        ThemeConfig {
            name: "Rosé Pine Moon".to_string(),
            description: "Medium-tone moonlight Rosé Pine variant".to_string(),

            bg: "#232136".to_string(),
            fg: "#e0def4".to_string(),
            fg_dim: "#908caa".to_string(),
            fg_subtle: "#6e6a86".to_string(),
            accent: "#ea9a97".to_string(),
            accent_subtle: "#2a273f".to_string(),
            selection_bg: "#393552".to_string(),
            selection_fg: "#e0def4".to_string(),

            border_active: "#ea9a97".to_string(),
            border_inactive: "#2a273f".to_string(),
            border_accent: "#f6c177".to_string(),

            sidebar_bg: "#2a273f".to_string(),
            sidebar_header_fg: "#6e6a86".to_string(),
            sidebar_folder_fg: "#ea9a97".to_string(),
            sidebar_feed_fg: "#e0def4".to_string(),
            sidebar_unread_badge_bg: "#393552".to_string(),
            sidebar_unread_badge_fg: "#eb6f92".to_string(),

            article_list_bg: "#232136".to_string(),
            article_card_bg: "#2a273f".to_string(),
            article_card_selected_bg: "#393552".to_string(),
            article_card_selected_border: "#ea9a97".to_string(),
            article_title_unread: "#e0def4".to_string(),
            article_title_read: "#908caa".to_string(),
            article_summary_fg: "#908caa".to_string(),
            article_meta_fg: "#9ccfd8".to_string(),
            article_unread_dot: "#eb6f92".to_string(),
            article_star: "#f6c177".to_string(),

            reader_bg: "#232136".to_string(),
            reader_header_feed: "#ea9a97".to_string(),
            reader_header_author: "#908caa".to_string(),
            reader_title: "#e0def4".to_string(),
            reader_meta: "#6e6a86".to_string(),
            reader_divider: "#393552".to_string(),
            reader_body: "#e0def4".to_string(),
            reader_h1: "#c4a7e7".to_string(),
            reader_h2: "#ea9a97".to_string(),
            reader_h3: "#9ccfd8".to_string(),
            reader_quote_border: "#eb6f92".to_string(),
            reader_quote_fg: "#908caa".to_string(),
            reader_code_bg: "#2a273f".to_string(),
            reader_code_fg: "#3e8fb0".to_string(),
            reader_link: "#9ccfd8".to_string(),
            reader_link_url: "#6e6a86".to_string(),

            status_bar_bg: "#2a273f".to_string(),
            status_bar_fg: "#908caa".to_string(),
            modal_bg: "#232136".to_string(),
            modal_border: "#ea9a97".to_string(),
            error_fg: "#eb6f92".to_string(),
            success_fg: "#9ccfd8".to_string(),
            warning_fg: "#f6c177".to_string(),
        }
        .into()
    }

    pub fn monokai_pro() -> Self {
        ThemeConfig {
            name: "Monokai Pro".to_string(),
            description: "Vibrant spectrum and high contrast pro palette".to_string(),

            bg: "#2d2a2e".to_string(),
            fg: "#fcfcfa".to_string(),
            fg_dim: "#939293".to_string(),
            fg_subtle: "#5b595c".to_string(),
            accent: "#ffd866".to_string(),
            accent_subtle: "#403e41".to_string(),
            selection_bg: "#403e41".to_string(),
            selection_fg: "#fcfcfa".to_string(),

            border_active: "#ffd866".to_string(),
            border_inactive: "#403e41".to_string(),
            border_accent: "#ff6188".to_string(),

            sidebar_bg: "#221f22".to_string(),
            sidebar_header_fg: "#727072".to_string(),
            sidebar_folder_fg: "#ffd866".to_string(),
            sidebar_feed_fg: "#fcfcfa".to_string(),
            sidebar_unread_badge_bg: "#403e41".to_string(),
            sidebar_unread_badge_fg: "#ff6188".to_string(),

            article_list_bg: "#2d2a2e".to_string(),
            article_card_bg: "#221f22".to_string(),
            article_card_selected_bg: "#403e41".to_string(),
            article_card_selected_border: "#ffd866".to_string(),
            article_title_unread: "#fcfcfa".to_string(),
            article_title_read: "#727072".to_string(),
            article_summary_fg: "#939293".to_string(),
            article_meta_fg: "#78dce8".to_string(),
            article_unread_dot: "#ff6188".to_string(),
            article_star: "#ffd866".to_string(),

            reader_bg: "#2d2a2e".to_string(),
            reader_header_feed: "#ffd866".to_string(),
            reader_header_author: "#939293".to_string(),
            reader_title: "#fcfcfa".to_string(),
            reader_meta: "#727072".to_string(),
            reader_divider: "#403e41".to_string(),
            reader_body: "#fcfcfa".to_string(),
            reader_h1: "#ff6188".to_string(),
            reader_h2: "#fc9867".to_string(),
            reader_h3: "#ffd866".to_string(),
            reader_quote_border: "#a9dc76".to_string(),
            reader_quote_fg: "#939293".to_string(),
            reader_code_bg: "#221f22".to_string(),
            reader_code_fg: "#a9dc76".to_string(),
            reader_link: "#78dce8".to_string(),
            reader_link_url: "#727072".to_string(),

            status_bar_bg: "#221f22".to_string(),
            status_bar_fg: "#939293".to_string(),
            modal_bg: "#2d2a2e".to_string(),
            modal_border: "#ffd866".to_string(),
            error_fg: "#ff6188".to_string(),
            success_fg: "#a9dc76".to_string(),
            warning_fg: "#ffd866".to_string(),
        }
        .into()
    }

    pub fn one_dark() -> Self {
        ThemeConfig {
            name: "One Dark".to_string(),
            description: "Iconic Atom / VS Code modern dark theme".to_string(),

            bg: "#282c34".to_string(),
            fg: "#abb2bf".to_string(),
            fg_dim: "#5c6370".to_string(),
            fg_subtle: "#4b5263".to_string(),
            accent: "#61afef".to_string(),
            accent_subtle: "#2c313a".to_string(),
            selection_bg: "#3e4451".to_string(),
            selection_fg: "#ffffff".to_string(),

            border_active: "#61afef".to_string(),
            border_inactive: "#3e4451".to_string(),
            border_accent: "#e06c75".to_string(),

            sidebar_bg: "#21252b".to_string(),
            sidebar_header_fg: "#5c6370".to_string(),
            sidebar_folder_fg: "#61afef".to_string(),
            sidebar_feed_fg: "#abb2bf".to_string(),
            sidebar_unread_badge_bg: "#2c313a".to_string(),
            sidebar_unread_badge_fg: "#61afef".to_string(),

            article_list_bg: "#282c34".to_string(),
            article_card_bg: "#21252b".to_string(),
            article_card_selected_bg: "#3e4451".to_string(),
            article_card_selected_border: "#61afef".to_string(),
            article_title_unread: "#ffffff".to_string(),
            article_title_read: "#5c6370".to_string(),
            article_summary_fg: "#abb2bf".to_string(),
            article_meta_fg: "#56b6c2".to_string(),
            article_unread_dot: "#61afef".to_string(),
            article_star: "#e5c07b".to_string(),

            reader_bg: "#282c34".to_string(),
            reader_header_feed: "#61afef".to_string(),
            reader_header_author: "#5c6370".to_string(),
            reader_title: "#ffffff".to_string(),
            reader_meta: "#5c6370".to_string(),
            reader_divider: "#3e4451".to_string(),
            reader_body: "#abb2bf".to_string(),
            reader_h1: "#c678dd".to_string(),
            reader_h2: "#61afef".to_string(),
            reader_h3: "#56b6c2".to_string(),
            reader_quote_border: "#e06c75".to_string(),
            reader_quote_fg: "#5c6370".to_string(),
            reader_code_bg: "#21252b".to_string(),
            reader_code_fg: "#98c379".to_string(),
            reader_link: "#56b6c2".to_string(),
            reader_link_url: "#4b5263".to_string(),

            status_bar_bg: "#21252b".to_string(),
            status_bar_fg: "#5c6370".to_string(),
            modal_bg: "#282c34".to_string(),
            modal_border: "#61afef".to_string(),
            error_fg: "#e06c75".to_string(),
            success_fg: "#98c379".to_string(),
            warning_fg: "#e5c07b".to_string(),
        }
        .into()
    }

    pub fn github_dark() -> Self {
        ThemeConfig {
            name: "GitHub Dark".to_string(),
            description: "Official GitHub modern dark theme".to_string(),

            bg: "#0d1117".to_string(),
            fg: "#e6edf3".to_string(),
            fg_dim: "#7d8590".to_string(),
            fg_subtle: "#484f58".to_string(),
            accent: "#2f81f7".to_string(),
            accent_subtle: "#161b22".to_string(),
            selection_bg: "#1f242c".to_string(),
            selection_fg: "#ffffff".to_string(),

            border_active: "#2f81f7".to_string(),
            border_inactive: "#30363d".to_string(),
            border_accent: "#f0883e".to_string(),

            sidebar_bg: "#010409".to_string(),
            sidebar_header_fg: "#7d8590".to_string(),
            sidebar_folder_fg: "#2f81f7".to_string(),
            sidebar_feed_fg: "#c9d1d9".to_string(),
            sidebar_unread_badge_bg: "#161b22".to_string(),
            sidebar_unread_badge_fg: "#58a6ff".to_string(),

            article_list_bg: "#0d1117".to_string(),
            article_card_bg: "#161b22".to_string(),
            article_card_selected_bg: "#1f242c".to_string(),
            article_card_selected_border: "#2f81f7".to_string(),
            article_title_unread: "#ffffff".to_string(),
            article_title_read: "#7d8590".to_string(),
            article_summary_fg: "#8b949e".to_string(),
            article_meta_fg: "#58a6ff".to_string(),
            article_unread_dot: "#2f81f7".to_string(),
            article_star: "#d29922".to_string(),

            reader_bg: "#0d1117".to_string(),
            reader_header_feed: "#58a6ff".to_string(),
            reader_header_author: "#8b949e".to_string(),
            reader_title: "#ffffff".to_string(),
            reader_meta: "#484f58".to_string(),
            reader_divider: "#30363d".to_string(),
            reader_body: "#e6edf3".to_string(),
            reader_h1: "#58a6ff".to_string(),
            reader_h2: "#79c0ff".to_string(),
            reader_h3: "#a5d6ff".to_string(),
            reader_quote_border: "#388bfd".to_string(),
            reader_quote_fg: "#8b949e".to_string(),
            reader_code_bg: "#161b22".to_string(),
            reader_code_fg: "#7ee787".to_string(),
            reader_link: "#58a6ff".to_string(),
            reader_link_url: "#484f58".to_string(),

            status_bar_bg: "#010409".to_string(),
            status_bar_fg: "#7d8590".to_string(),
            modal_bg: "#161b22".to_string(),
            modal_border: "#2f81f7".to_string(),
            error_fg: "#f85149".to_string(),
            success_fg: "#3fb950".to_string(),
            warning_fg: "#d29922".to_string(),
        }
        .into()
    }

    pub fn github_light() -> Self {
        ThemeConfig {
            name: "GitHub Light".to_string(),
            description: "Official GitHub clean light theme".to_string(),

            bg: "#ffffff".to_string(),
            fg: "#1f2328".to_string(),
            fg_dim: "#656d76".to_string(),
            fg_subtle: "#8c959f".to_string(),
            accent: "#0969da".to_string(),
            accent_subtle: "#f6f8fa".to_string(),
            selection_bg: "#ddf4ff".to_string(),
            selection_fg: "#0969da".to_string(),

            border_active: "#0969da".to_string(),
            border_inactive: "#d0d7de".to_string(),
            border_accent: "#bc4c00".to_string(),

            sidebar_bg: "#f6f8fa".to_string(),
            sidebar_header_fg: "#656d76".to_string(),
            sidebar_folder_fg: "#0969da".to_string(),
            sidebar_feed_fg: "#1f2328".to_string(),
            sidebar_unread_badge_bg: "#eaeef2".to_string(),
            sidebar_unread_badge_fg: "#0969da".to_string(),

            article_list_bg: "#ffffff".to_string(),
            article_card_bg: "#f6f8fa".to_string(),
            article_card_selected_bg: "#ddf4ff".to_string(),
            article_card_selected_border: "#0969da".to_string(),
            article_title_unread: "#1f2328".to_string(),
            article_title_read: "#656d76".to_string(),
            article_summary_fg: "#656d76".to_string(),
            article_meta_fg: "#0969da".to_string(),
            article_unread_dot: "#0969da".to_string(),
            article_star: "#9a6700".to_string(),

            reader_bg: "#ffffff".to_string(),
            reader_header_feed: "#0969da".to_string(),
            reader_header_author: "#656d76".to_string(),
            reader_title: "#1f2328".to_string(),
            reader_meta: "#8c959f".to_string(),
            reader_divider: "#d0d7de".to_string(),
            reader_body: "#1f2328".to_string(),
            reader_h1: "#0969da".to_string(),
            reader_h2: "#0550ae".to_string(),
            reader_h3: "#0a3069".to_string(),
            reader_quote_border: "#0969da".to_string(),
            reader_quote_fg: "#656d76".to_string(),
            reader_code_bg: "#f6f8fa".to_string(),
            reader_code_fg: "#1a7f37".to_string(),
            reader_link: "#0969da".to_string(),
            reader_link_url: "#8c959f".to_string(),

            status_bar_bg: "#f6f8fa".to_string(),
            status_bar_fg: "#656d76".to_string(),
            modal_bg: "#ffffff".to_string(),
            modal_border: "#0969da".to_string(),
            error_fg: "#cf222e".to_string(),
            success_fg: "#1a7f37".to_string(),
            warning_fg: "#9a6700".to_string(),
        }
        .into()
    }

    pub fn kanagawa() -> Self {
        ThemeConfig {
            name: "Kanagawa".to_string(),
            description: "Traditional Japanese ink & woodblock print colors".to_string(),

            bg: "#1f1f28".to_string(),
            fg: "#dcd7ba".to_string(),
            fg_dim: "#938aa9".to_string(),
            fg_subtle: "#54546d".to_string(),
            accent: "#7e9cd8".to_string(),
            accent_subtle: "#2a2a37".to_string(),
            selection_bg: "#2d4f67".to_string(),
            selection_fg: "#dcd7ba".to_string(),

            border_active: "#7e9cd8".to_string(),
            border_inactive: "#2a2a37".to_string(),
            border_accent: "#ffa066".to_string(),

            sidebar_bg: "#16161d".to_string(),
            sidebar_header_fg: "#727169".to_string(),
            sidebar_folder_fg: "#7e9cd8".to_string(),
            sidebar_feed_fg: "#c8c093".to_string(),
            sidebar_unread_badge_bg: "#2a2a37".to_string(),
            sidebar_unread_badge_fg: "#7e9cd8".to_string(),

            article_list_bg: "#1f1f28".to_string(),
            article_card_bg: "#16161d".to_string(),
            article_card_selected_bg: "#223249".to_string(),
            article_card_selected_border: "#7e9cd8".to_string(),
            article_title_unread: "#dcd7ba".to_string(),
            article_title_read: "#727169".to_string(),
            article_summary_fg: "#938aa9".to_string(),
            article_meta_fg: "#7aa89f".to_string(),
            article_unread_dot: "#e82424".to_string(),
            article_star: "#dca561".to_string(),

            reader_bg: "#1f1f28".to_string(),
            reader_header_feed: "#7e9cd8".to_string(),
            reader_header_author: "#938aa9".to_string(),
            reader_title: "#dcd7ba".to_string(),
            reader_meta: "#54546d".to_string(),
            reader_divider: "#2a2a37".to_string(),
            reader_body: "#dcd7ba".to_string(),
            reader_h1: "#957fb8".to_string(),
            reader_h2: "#7e9cd8".to_string(),
            reader_h3: "#7aa89f".to_string(),
            reader_quote_border: "#e46876".to_string(),
            reader_quote_fg: "#938aa9".to_string(),
            reader_code_bg: "#16161d".to_string(),
            reader_code_fg: "#98bb6c".to_string(),
            reader_link: "#7fb4ca".to_string(),
            reader_link_url: "#54546d".to_string(),

            status_bar_bg: "#16161d".to_string(),
            status_bar_fg: "#727169".to_string(),
            modal_bg: "#1f1f28".to_string(),
            modal_border: "#7e9cd8".to_string(),
            error_fg: "#e82424".to_string(),
            success_fg: "#98bb6c".to_string(),
            warning_fg: "#dca561".to_string(),
        }
        .into()
    }

    pub fn everforest_dark() -> Self {
        ThemeConfig {
            name: "Everforest Dark".to_string(),
            description: "Soothing natural organic green-tinted dark theme".to_string(),

            bg: "#2d353b".to_string(),
            fg: "#d3c6aa".to_string(),
            fg_dim: "#9da9a0".to_string(),
            fg_subtle: "#5c6a72".to_string(),
            accent: "#a7c080".to_string(),
            accent_subtle: "#343f44".to_string(),
            selection_bg: "#3d484d".to_string(),
            selection_fg: "#d3c6aa".to_string(),

            border_active: "#a7c080".to_string(),
            border_inactive: "#3d484d".to_string(),
            border_accent: "#e69875".to_string(),

            sidebar_bg: "#232a2e".to_string(),
            sidebar_header_fg: "#7a8478".to_string(),
            sidebar_folder_fg: "#a7c080".to_string(),
            sidebar_feed_fg: "#d3c6aa".to_string(),
            sidebar_unread_badge_bg: "#343f44".to_string(),
            sidebar_unread_badge_fg: "#a7c080".to_string(),

            article_list_bg: "#2d353b".to_string(),
            article_card_bg: "#232a2e".to_string(),
            article_card_selected_bg: "#3d484d".to_string(),
            article_card_selected_border: "#a7c080".to_string(),
            article_title_unread: "#d3c6aa".to_string(),
            article_title_read: "#7a8478".to_string(),
            article_summary_fg: "#9da9a0".to_string(),
            article_meta_fg: "#7fbbb3".to_string(),
            article_unread_dot: "#a7c080".to_string(),
            article_star: "#dbbc7f".to_string(),

            reader_bg: "#2d353b".to_string(),
            reader_header_feed: "#a7c080".to_string(),
            reader_header_author: "#9da9a0".to_string(),
            reader_title: "#d3c6aa".to_string(),
            reader_meta: "#5c6a72".to_string(),
            reader_divider: "#3d484d".to_string(),
            reader_body: "#d3c6aa".to_string(),
            reader_h1: "#d699b6".to_string(),
            reader_h2: "#a7c080".to_string(),
            reader_h3: "#7fbbb3".to_string(),
            reader_quote_border: "#e67e80".to_string(),
            reader_quote_fg: "#9da9a0".to_string(),
            reader_code_bg: "#232a2e".to_string(),
            reader_code_fg: "#a7c080".to_string(),
            reader_link: "#7fbbb3".to_string(),
            reader_link_url: "#5c6a72".to_string(),

            status_bar_bg: "#232a2e".to_string(),
            status_bar_fg: "#9da9a0".to_string(),
            modal_bg: "#2d353b".to_string(),
            modal_border: "#a7c080".to_string(),
            error_fg: "#e67e80".to_string(),
            success_fg: "#a7c080".to_string(),
            warning_fg: "#dbbc7f".to_string(),
        }
        .into()
    }

    pub fn everforest_light() -> Self {
        ThemeConfig {
            name: "Everforest Light".to_string(),
            description: "Soothing natural organic green-tinted light theme".to_string(),

            bg: "#fdf6e3".to_string(),
            fg: "#5c6a72".to_string(),
            fg_dim: "#7a8478".to_string(),
            fg_subtle: "#939f91".to_string(),
            accent: "#8da101".to_string(),
            accent_subtle: "#f4f0d9".to_string(),
            selection_bg: "#eaedc8".to_string(),
            selection_fg: "#3a4d39".to_string(),

            border_active: "#8da101".to_string(),
            border_inactive: "#e0dcc7".to_string(),
            border_accent: "#f57d26".to_string(),

            sidebar_bg: "#f4f0d9".to_string(),
            sidebar_header_fg: "#7a8478".to_string(),
            sidebar_folder_fg: "#8da101".to_string(),
            sidebar_feed_fg: "#5c6a72".to_string(),
            sidebar_unread_badge_bg: "#e0dcc7".to_string(),
            sidebar_unread_badge_fg: "#8da101".to_string(),

            article_list_bg: "#fdf6e3".to_string(),
            article_card_bg: "#f4f0d9".to_string(),
            article_card_selected_bg: "#eaedc8".to_string(),
            article_card_selected_border: "#8da101".to_string(),
            article_title_unread: "#272e33".to_string(),
            article_title_read: "#829181".to_string(),
            article_summary_fg: "#5c6a72".to_string(),
            article_meta_fg: "#3a94c5".to_string(),
            article_unread_dot: "#8da101".to_string(),
            article_star: "#dfa000".to_string(),

            reader_bg: "#fdf6e3".to_string(),
            reader_header_feed: "#8da101".to_string(),
            reader_header_author: "#7a8478".to_string(),
            reader_title: "#272e33".to_string(),
            reader_meta: "#939f91".to_string(),
            reader_divider: "#e0dcc7".to_string(),
            reader_body: "#4c555b".to_string(),
            reader_h1: "#df69ba".to_string(),
            reader_h2: "#8da101".to_string(),
            reader_h3: "#3a94c5".to_string(),
            reader_quote_border: "#f85552".to_string(),
            reader_quote_fg: "#7a8478".to_string(),
            reader_code_bg: "#f4f0d9".to_string(),
            reader_code_fg: "#8da101".to_string(),
            reader_link: "#3a94c5".to_string(),
            reader_link_url: "#939f91".to_string(),

            status_bar_bg: "#f4f0d9".to_string(),
            status_bar_fg: "#5c6a72".to_string(),
            modal_bg: "#fdf6e3".to_string(),
            modal_border: "#8da101".to_string(),
            error_fg: "#f85552".to_string(),
            success_fg: "#8da101".to_string(),
            warning_fg: "#dfa000".to_string(),
        }
        .into()
    }

    pub fn cyberpunk_neon() -> Self {
        ThemeConfig {
            name: "Cyberpunk Neon".to_string(),
            description: "High-contrast synthwave neon cyberpunk palette".to_string(),

            bg: "#05050f".to_string(),
            fg: "#f0f0ff".to_string(),
            fg_dim: "#8888b0".to_string(),
            fg_subtle: "#444470".to_string(),
            accent: "#00ffcc".to_string(),
            accent_subtle: "#12112a".to_string(),
            selection_bg: "#24183a".to_string(),
            selection_fg: "#00ffcc".to_string(),

            border_active: "#ff007f".to_string(),
            border_inactive: "#20183b".to_string(),
            border_accent: "#00ffcc".to_string(),

            sidebar_bg: "#080616".to_string(),
            sidebar_header_fg: "#8888b0".to_string(),
            sidebar_folder_fg: "#ff007f".to_string(),
            sidebar_feed_fg: "#f0f0ff".to_string(),
            sidebar_unread_badge_bg: "#24183a".to_string(),
            sidebar_unread_badge_fg: "#00ffcc".to_string(),

            article_list_bg: "#05050f".to_string(),
            article_card_bg: "#0e0c22".to_string(),
            article_card_selected_bg: "#24183a".to_string(),
            article_card_selected_border: "#00ffcc".to_string(),
            article_title_unread: "#ffffff".to_string(),
            article_title_read: "#66668a".to_string(),
            article_summary_fg: "#b8b8e0".to_string(),
            article_meta_fg: "#00e5ff".to_string(),
            article_unread_dot: "#ff007f".to_string(),
            article_star: "#ffe600".to_string(),

            reader_bg: "#05050f".to_string(),
            reader_header_feed: "#00ffcc".to_string(),
            reader_header_author: "#8888b0".to_string(),
            reader_title: "#ffffff".to_string(),
            reader_meta: "#555580".to_string(),
            reader_divider: "#20183b".to_string(),
            reader_body: "#f0f0ff".to_string(),
            reader_h1: "#ff007f".to_string(),
            reader_h2: "#00ffcc".to_string(),
            reader_h3: "#ffe600".to_string(),
            reader_quote_border: "#ff007f".to_string(),
            reader_quote_fg: "#8888b0".to_string(),
            reader_code_bg: "#0e0c22".to_string(),
            reader_code_fg: "#00ffcc".to_string(),
            reader_link: "#00e5ff".to_string(),
            reader_link_url: "#66668a".to_string(),

            status_bar_bg: "#080616".to_string(),
            status_bar_fg: "#8888b0".to_string(),
            modal_bg: "#0e0c22".to_string(),
            modal_border: "#00ffcc".to_string(),
            error_fg: "#ff0055".to_string(),
            success_fg: "#00ffcc".to_string(),
            warning_fg: "#ffe600".to_string(),
        }
        .into()
    }

    pub fn horizon() -> Self {
        ThemeConfig {
            name: "Horizon".to_string(),
            description: "Warm dusk dual-tone neon purple and apricot theme".to_string(),

            bg: "#1c1e26".to_string(),
            fg: "#d5d8da".to_string(),
            fg_dim: "#9da0a2".to_string(),
            fg_subtle: "#6c6f71".to_string(),
            accent: "#e95678".to_string(),
            accent_subtle: "#232530".to_string(),
            selection_bg: "#2e303e".to_string(),
            selection_fg: "#fab795".to_string(),

            border_active: "#e95678".to_string(),
            border_inactive: "#2e303e".to_string(),
            border_accent: "#fab795".to_string(),

            sidebar_bg: "#16171d".to_string(),
            sidebar_header_fg: "#6c6f71".to_string(),
            sidebar_folder_fg: "#e95678".to_string(),
            sidebar_feed_fg: "#d5d8da".to_string(),
            sidebar_unread_badge_bg: "#2e303e".to_string(),
            sidebar_unread_badge_fg: "#fab795".to_string(),

            article_list_bg: "#1c1e26".to_string(),
            article_card_bg: "#16171d".to_string(),
            article_card_selected_bg: "#2e303e".to_string(),
            article_card_selected_border: "#e95678".to_string(),
            article_title_unread: "#fdfdfd".to_string(),
            article_title_read: "#6c6f71".to_string(),
            article_summary_fg: "#9da0a2".to_string(),
            article_meta_fg: "#26bbd9".to_string(),
            article_unread_dot: "#e95678".to_string(),
            article_star: "#fac29a".to_string(),

            reader_bg: "#1c1e26".to_string(),
            reader_header_feed: "#e95678".to_string(),
            reader_header_author: "#9da0a2".to_string(),
            reader_title: "#fdfdfd".to_string(),
            reader_meta: "#6c6f71".to_string(),
            reader_divider: "#2e303e".to_string(),
            reader_body: "#d5d8da".to_string(),
            reader_h1: "#b877db".to_string(),
            reader_h2: "#e95678".to_string(),
            reader_h3: "#fab795".to_string(),
            reader_quote_border: "#e95678".to_string(),
            reader_quote_fg: "#9da0a2".to_string(),
            reader_code_bg: "#16171d".to_string(),
            reader_code_fg: "#29d398".to_string(),
            reader_link: "#26bbd9".to_string(),
            reader_link_url: "#6c6f71".to_string(),

            status_bar_bg: "#16171d".to_string(),
            status_bar_fg: "#9da0a2".to_string(),
            modal_bg: "#1c1e26".to_string(),
            modal_border: "#e95678".to_string(),
            error_fg: "#e95678".to_string(),
            success_fg: "#29d398".to_string(),
            warning_fg: "#fac29a".to_string(),
        }
        .into()
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
        }
        .into()
    }


    // ---- Palette-defined themes ----------------------------------------
    //
    // Ordered dark first; `all_presets` sorts light ones to the bottom by
    // measuring the background, so a theme's position here only affects its
    // order within its own group.

    pub fn ayu_dark() -> Self {
        Palette {
            name: "Ayu Dark",
            description: "Deep ink background with warm amber highlights",
            bg: "#0b0e14", bg_alt: "#0d1017", bg_sel: "#1b1f2b",
            fg: "#bfbdb6", fg_dim: "#9a9791", fg_subtle: "#565b66",
            accent: "#ffb454", red: "#f07178", green: "#aad94c",
            yellow: "#ffb454", blue: "#59c2ff", magenta: "#d2a6ff", cyan: "#39bae6",
        }
        .into()
    }

    pub fn ayu_mirage() -> Self {
        Palette {
            name: "Ayu Mirage",
            description: "Softer slate variant of Ayu, easy on the eyes",
            bg: "#1f2430", bg_alt: "#1a1f29", bg_sel: "#2d3440",
            fg: "#cccac2", fg_dim: "#a6a29a", fg_subtle: "#707a8c",
            accent: "#ffcc66", red: "#f28779", green: "#d5ff80",
            yellow: "#ffcc66", blue: "#73d0ff", magenta: "#dfbfff", cyan: "#5ccfe6",
        }
        .into()
    }

    pub fn material_ocean() -> Self {
        Palette {
            name: "Material Ocean",
            description: "Deep ocean blues from the Material palette",
            bg: "#0f111a", bg_alt: "#090b10", bg_sel: "#1f2233",
            fg: "#a6accd", fg_dim: "#8f96b3", fg_subtle: "#4b526d",
            accent: "#89ddff", red: "#f07178", green: "#c3e88d",
            yellow: "#ffcb6b", blue: "#82aaff", magenta: "#c792ea", cyan: "#89ddff",
        }
        .into()
    }

    pub fn zenburn() -> Self {
        Palette {
            name: "Zenburn",
            description: "Low-contrast muted classic, kind at 2am",
            bg: "#3f3f3f", bg_alt: "#383838", bg_sel: "#4f4f4f",
            fg: "#dcdccc", fg_dim: "#c0c0a8", fg_subtle: "#8f8f7f",
            accent: "#f0dfaf", red: "#cc9393", green: "#7f9f7f",
            yellow: "#f0dfaf", blue: "#8cd0d3", magenta: "#dc8cc3", cyan: "#93e0e3",
        }
        .into()
    }

    pub fn iceberg() -> Self {
        Palette {
            name: "Iceberg",
            description: "Cold blue-grey with restrained accents",
            bg: "#161821", bg_alt: "#0f1117", bg_sel: "#272c42",
            fg: "#c6c8d1", fg_dim: "#a3a6b3", fg_subtle: "#6b7089",
            accent: "#84a0c6", red: "#e27878", green: "#b4be82",
            yellow: "#e2a478", blue: "#84a0c6", magenta: "#a093c7", cyan: "#89b8c2",
        }
        .into()
    }

    pub fn oxocarbon() -> Self {
        Palette {
            name: "Oxocarbon",
            description: "IBM Carbon-derived near-black with vivid accents",
            bg: "#161616", bg_alt: "#0d0d0d", bg_sel: "#262626",
            fg: "#f2f4f8", fg_dim: "#c6c6c6", fg_subtle: "#6f6f6f",
            accent: "#33b1ff", red: "#ee5396", green: "#42be65",
            yellow: "#ffe97b", blue: "#33b1ff", magenta: "#be95ff", cyan: "#3ddbd9",
        }
        .into()
    }

    pub fn melange_dark() -> Self {
        Palette {
            name: "Melange Dark",
            description: "Warm cocoa browns, low glare",
            bg: "#292522", bg_alt: "#211f1c", bg_sel: "#403a35",
            fg: "#ece1d7", fg_dim: "#c1a78e", fg_subtle: "#867462",
            accent: "#d3a94a", red: "#d47766", green: "#85b695",
            yellow: "#ebc06d", blue: "#a3a9ce", magenta: "#cf9bc2", cyan: "#89b3b6",
        }
        .into()
    }

    pub fn poimandres() -> Self {
        Palette {
            name: "Poimandres",
            description: "Muted teal-on-navy, minimal and calm",
            bg: "#1b1e28", bg_alt: "#171922", bg_sel: "#282c39",
            fg: "#a6accd", fg_dim: "#8f95b2", fg_subtle: "#506477",
            accent: "#5de4c7", red: "#d0679d", green: "#5de4c7",
            yellow: "#fffac2", blue: "#89ddff", magenta: "#fcc5e9", cyan: "#add7ff",
        }
        .into()
    }

    pub fn vesper() -> Self {
        Palette {
            name: "Vesper",
            description: "Near-monochrome dark with a single warm accent",
            bg: "#101010", bg_alt: "#0a0a0a", bg_sel: "#232323",
            fg: "#ffffff", fg_dim: "#a0a0a0", fg_subtle: "#505050",
            accent: "#ffc799", red: "#ff8080", green: "#99ffe4",
            yellow: "#ffc799", blue: "#a0a0a0", magenta: "#ffcfa8", cyan: "#99ffe4",
        }
        .into()
    }

    pub fn flexoki_dark() -> Self {
        Palette {
            name: "Flexoki Dark",
            description: "Inky paper-inspired palette tuned for reading",
            bg: "#100f0f", bg_alt: "#1c1b1a", bg_sel: "#282726",
            fg: "#cecdc3", fg_dim: "#b7b5ac", fg_subtle: "#6f6e69",
            accent: "#d0a215", red: "#d14d41", green: "#879a39",
            yellow: "#d0a215", blue: "#4385be", magenta: "#ce5d97", cyan: "#3aa99f",
        }
        .into()
    }

    pub fn tokyo_night_moon() -> Self {
        Palette {
            name: "Tokyo Night Moon",
            description: "Lighter, bluer sibling of Tokyo Night",
            bg: "#222436", bg_alt: "#1e2030", bg_sel: "#2f334d",
            fg: "#c8d3f5", fg_dim: "#a9b8e8", fg_subtle: "#636da6",
            accent: "#82aaff", red: "#ff757f", green: "#c3e88d",
            yellow: "#ffc777", blue: "#82aaff", magenta: "#c099ff", cyan: "#86e1fc",
        }
        .into()
    }

    pub fn gruvbox_material() -> Self {
        Palette {
            name: "Gruvbox Material",
            description: "Softened Gruvbox with lower saturation",
            bg: "#282828", bg_alt: "#1d2021", bg_sel: "#3c3836",
            fg: "#d4be98", fg_dim: "#bdae93", fg_subtle: "#928374",
            accent: "#a9b665", red: "#ea6962", green: "#a9b665",
            yellow: "#d8a657", blue: "#7daea3", magenta: "#d3869b", cyan: "#89b482",
        }
        .into()
    }

    pub fn nightfox() -> Self {
        Palette {
            name: "Nightfox",
            description: "Balanced dark blue with clear syntax hues",
            bg: "#192330", bg_alt: "#131a24", bg_sel: "#29394f",
            fg: "#cdcecf", fg_dim: "#aeafb0", fg_subtle: "#71839b",
            accent: "#719cd6", red: "#c94f6d", green: "#81b29a",
            yellow: "#dbc074", blue: "#719cd6", magenta: "#9d79d6", cyan: "#63cdcf",
        }
        .into()
    }

    // ---- Light themes ---------------------------------------------------

    pub fn ayu_light() -> Self {
        Palette {
            name: "Ayu Light",
            description: "Crisp warm white with amber accents",
            bg: "#fcfcfc", bg_alt: "#f3f4f5", bg_sel: "#e7e8e9",
            fg: "#5c6166", fg_dim: "#787b80", fg_subtle: "#8a8f98",
            accent: "#ff9940", red: "#e65050", green: "#6cbf43",
            yellow: "#f2ae49", blue: "#399ee6", magenta: "#a37acc", cyan: "#55b4d4",
        }
        .into()
    }

    pub fn papercolor_light() -> Self {
        Palette {
            name: "PaperColor Light",
            description: "High-contrast paper white, strong primaries",
            bg: "#eeeeee", bg_alt: "#e4e4e4", bg_sel: "#d0d0d0",
            fg: "#444444", fg_dim: "#5f5f5f", fg_subtle: "#878787",
            accent: "#0087af", red: "#af0000", green: "#008700",
            yellow: "#d75f00", blue: "#0087af", magenta: "#8700af", cyan: "#005f87",
        }
        .into()
    }

    pub fn flexoki_light() -> Self {
        Palette {
            name: "Flexoki Light",
            description: "Warm paper tone designed for long reading",
            bg: "#fffcf0", bg_alt: "#f2f0e5", bg_sel: "#e6e4d9",
            fg: "#100f0f", fg_dim: "#403e3c", fg_subtle: "#878580",
            accent: "#bc5215", red: "#af3029", green: "#66800b",
            yellow: "#ad8301", blue: "#205ea6", magenta: "#a02f6f", cyan: "#24837b",
        }
        .into()
    }

    pub fn melange_light() -> Self {
        Palette {
            name: "Melange Light",
            description: "Soft sand tones, the light side of Melange",
            bg: "#f1f1f1", bg_alt: "#e9e1db", bg_sel: "#dcd3cd",
            fg: "#54433a", fg_dim: "#6b5a50", fg_subtle: "#a98a78",
            accent: "#a06d00", red: "#c77b8b", green: "#6e9b72",
            yellow: "#bc5c00", blue: "#7892bd", magenta: "#be79bb", cyan: "#739797",
        }
        .into()
    }

    pub fn tokyo_night_day() -> Self {
        Palette {
            name: "Tokyo Night Day",
            description: "Daylight counterpart to Tokyo Night",
            bg: "#e1e2e7", bg_alt: "#d0d5e3", bg_sel: "#c4c8da",
            fg: "#3760bf", fg_dim: "#535f89", fg_subtle: "#848cb5",
            accent: "#2e7de9", red: "#f52a65", green: "#587539",
            yellow: "#8c6c3e", blue: "#2e7de9", magenta: "#9854f1", cyan: "#007197",
        }
        .into()
    }

    pub fn iceberg_light() -> Self {
        Palette {
            name: "Iceberg Light",
            description: "Pale blue-grey, the light Iceberg variant",
            bg: "#e8e9ec", bg_alt: "#dcdfe7", bg_sel: "#cad0de",
            fg: "#33374c", fg_dim: "#575f78", fg_subtle: "#8389a3",
            accent: "#2d539e", red: "#cc517a", green: "#668e3d",
            yellow: "#c57339", blue: "#2d539e", magenta: "#7759b4", cyan: "#3f83a6",
        }
        .into()
    }

    pub fn all_presets() -> Vec<Theme> {
        let mut presets = vec![
            Self::ratarss_dark(),
            Self::ratarss_light(),
            Self::catppuccin_mocha(),
            Self::catppuccin_macchiato(),
            Self::catppuccin_frappe(),
            Self::catppuccin_latte(),
            Self::tokyo_night(),
            Self::tokyo_night_storm(),
            Self::gruvbox_dark(),
            Self::gruvbox_light(),
            Self::nord(),
            Self::dracula(),
            Self::solarized_dark(),
            Self::solarized_light(),
            Self::rose_pine(),
            Self::rose_pine_dawn(),
            Self::rose_pine_moon(),
            Self::monokai_pro(),
            Self::one_dark(),
            Self::github_dark(),
            Self::github_light(),
            Self::kanagawa(),
            Self::everforest_dark(),
            Self::everforest_light(),
            Self::cyberpunk_neon(),
            Self::horizon(),
            Self::minimal_mono(),
            Self::ayu_dark(),
            Self::ayu_mirage(),
            Self::material_ocean(),
            Self::zenburn(),
            Self::iceberg(),
            Self::oxocarbon(),
            Self::melange_dark(),
            Self::poimandres(),
            Self::vesper(),
            Self::flexoki_dark(),
            Self::tokyo_night_moon(),
            Self::gruvbox_material(),
            Self::nightfox(),
            Self::ayu_light(),
            Self::papercolor_light(),
            Self::flexoki_light(),
            Self::melange_light(),
            Self::tokyo_night_day(),
            Self::iceberg_light(),
        ];

        // Light themes go last. Judged by the background's luminance rather
        // than by name, so a theme is classified by how it actually looks and
        // nothing has to be kept in a parallel list.
        //
        // `sort_by_key` is stable, so the curated order within each group holds.
        presets.sort_by_key(|t| t.is_light());
        presets
    }

    /// Whether this theme reads as a light theme, from the perceived luminance
    /// of its background.
    pub fn is_light(&self) -> bool {
        fn channel(hex: &str, at: usize) -> f32 {
            u8::from_str_radix(hex.get(at..at + 2).unwrap_or("00"), 16).unwrap_or(0) as f32 / 255.0
        }
        let hex = self.config.bg.trim_start_matches('#');
        if hex.len() < 6 {
            return false;
        }
        // Rec. 709 luma; 0.5 splits every bundled palette correctly.
        let luma = 0.2126 * channel(hex, 0) + 0.7152 * channel(hex, 2) + 0.0722 * channel(hex, 4);
        luma > 0.5
    }

    pub fn by_name(name: &str) -> Self {
        if name.eq_ignore_ascii_case("NetNewsWire Dark") || name.eq_ignore_ascii_case("RataRSS Dark") {
            return Self::ratarss_dark();
        }
        if name.eq_ignore_ascii_case("NetNewsWire Light") || name.eq_ignore_ascii_case("RataRSS Light") {
            return Self::ratarss_light();
        }
        for theme in Self::all_presets() {
            if theme.config.name.eq_ignore_ascii_case(name) {
                return theme;
            }
        }
        Self::ratarss_dark()
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
