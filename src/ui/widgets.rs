use crate::theme::Theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};


pub struct Badge<'a> {
    pub text: &'a str,
    pub bg_color: ratatui::style::Color,
    pub fg_color: ratatui::style::Color,
}

impl<'a> Badge<'a> {
    pub fn new(text: &'a str, theme: &Theme, is_selected: bool) -> Self {
        if is_selected {
            Self {
                text,
                bg_color: theme.accent,
                fg_color: theme.selection_fg,
            }
        } else {
            Self {
                text,
                bg_color: theme.sidebar_unread_badge_bg,
                fg_color: theme.sidebar_unread_badge_fg,
            }
        }
    }

    pub fn to_span(&self) -> Span<'static> {
        Span::styled(
            format!(" {} ", self.text),
            Style::default()
                .bg(self.bg_color)
                .fg(self.fg_color)
                .add_modifier(Modifier::BOLD),
        )
    }
}

#[allow(dead_code)]
pub struct ProgressBarWidget {
    pub progress: f64, // 0.0 to 1.0
    pub width: usize,
    pub theme: Theme,
}

#[allow(dead_code)]
impl ProgressBarWidget {
    pub fn to_line(&self) -> Line<'static> {
        let filled_chars = ((self.progress.clamp(0.0, 1.0) * self.width as f64).round() as usize).min(self.width);
        let empty_chars = self.width.saturating_sub(filled_chars);

        let percent = (self.progress * 100.0).round() as u64;

        Line::from(vec![
            Span::styled("[", Style::default().fg(self.theme.fg_subtle)),
            Span::styled("█".repeat(filled_chars), Style::default().fg(self.theme.accent)),
            Span::styled("░".repeat(empty_chars), Style::default().fg(self.theme.border_inactive)),
            Span::styled(format!("] {percent:>3}%"), Style::default().fg(self.theme.fg_dim)),
        ])
    }
}

pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn get_spinner_frame(tick: usize) -> &'static str {
    SPINNER_FRAMES[tick % SPINNER_FRAMES.len()]
}
