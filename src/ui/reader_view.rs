use crate::model::ActivePane;
use crate::reader::FormattedArticle;
use crate::theme::Theme;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Widget};

pub struct ReaderView<'a> {
    /// Pre-formatted article text, cached by the app between frames.
    pub formatted: Option<&'a FormattedArticle>,
    pub active_pane: ActivePane,
    pub theme: &'a Theme,
    pub scroll_offset: usize,
    pub is_zen_mode: bool,
    pub show_icons: bool,
    pub padding: u16,
}

impl<'a> ReaderView<'a> {
    /// The region article text is drawn into for a reader occupying `area`.
    ///
    /// Shared by the renderer and by click hit-testing, so a link can never be
    /// drawn in one place and clicked in another.
    pub fn text_area(area: Rect, padding: u16) -> Rect {
        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };
        let pad = padding.min(inner.width.saturating_sub(1) / 2);
        Rect {
            x: inner.x + pad,
            y: inner.y,
            width: inner.width.saturating_sub(pad * 2),
            // The last row carries the position indicator.
            height: inner.height.saturating_sub(1),
        }
    }

    /// Width available to article text in a reader occupying `area`.
    ///
    /// Used both to lay the text out and as part of the format cache key, so
    /// the cached lines always match the region they are drawn into — including
    /// after a padding change.
    pub fn inner_width(area: Rect, padding: u16) -> u16 {
        let inner = area.width.saturating_sub(2);
        let pad = padding.min(inner.saturating_sub(1) / 2);
        inner.saturating_sub(pad * 2)
    }
}

impl<'a> Widget for ReaderView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_focused = self.active_pane == ActivePane::Reader;
        let border_style = if is_focused {
            self.theme.active_border_style()
        } else {
            self.theme.inactive_border_style()
        };

        let mode_label = if self.is_zen_mode { " (f)" } else { "" };
        let title = if self.show_icons {
            format!(" 📖 Reader{mode_label} ")
        } else {
            format!(" Reader{mode_label} ")
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(Style::default().bg(self.theme.reader_bg))
            .title(Span::styled(
                title,
                if is_focused {
                    Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.fg_dim)
                },
            ));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.height == 0 || inner_area.width == 0 {
            return;
        }

        // Fill background in one call instead of one per cell.
        buf.set_style(inner_area, Style::default().bg(self.theme.reader_bg));

        let text = Self::text_area(area, self.padding);
        let (text_x, text_width) = (text.x, text.width);
        debug_assert_eq!(text_width, Self::inner_width(area, self.padding));

        let formatted = match self.formatted {
            Some(f) => f,
            None => {
                buf.set_string(
                    text_x,
                    inner_area.y + 1,
                    "No article selected.",
                    Style::default().fg(self.theme.fg_dim).add_modifier(Modifier::ITALIC),
                );
                return;
            }
        };

        let total_lines = formatted.total_lines;
        let visible_lines = inner_area.height.saturating_sub(1) as usize; // reserve 1 line for position indicator

        let start_line = self.scroll_offset.min(total_lines.saturating_sub(1));
        let end_line = (start_line + visible_lines).min(total_lines);

        for (idx, line_idx) in (start_line..end_line).enumerate() {
            let y = inner_area.y + idx as u16;
            if let Some(line) = formatted.lines.get(line_idx) {
                buf.set_line(text_x, y, line, text_width);
            }
        }

        // Bottom Position Indicator (e.g. Line 1/18)
        if total_lines > 0 && inner_area.height > 1 {
            let progress_y = inner_area.y + inner_area.height - 1;
            let current_pos_info = format!(" {}/{} ", start_line + 1, total_lines);
            let footer_width = unicode_width::UnicodeWidthStr::width(current_pos_info.as_str()) as u16;
            if inner_area.width > footer_width + 4 {
                let footer_x = inner_area.x + inner_area.width - footer_width - 1;
                buf.set_string(
                    footer_x,
                    progress_y,
                    &current_pos_info,
                    Style::default().fg(self.theme.fg_subtle),
                );
            }
        }
    }
}
