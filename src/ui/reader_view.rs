use crate::model::{ActivePane, Article};
use crate::reader::render_article_to_text;
use crate::theme::Theme;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};

pub struct ReaderView<'a> {
    pub article: Option<&'a Article>,
    pub active_pane: ActivePane,
    pub theme: &'a Theme,
    pub scroll_offset: usize,
    pub is_zen_mode: bool,
}

impl<'a> Widget for ReaderView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_focused = self.active_pane == ActivePane::Reader;
        let border_style = if is_focused {
            self.theme.active_border_style()
        } else {
            self.theme.inactive_border_style()
        };

        let mode_label = if self.is_zen_mode { " [FULLSCREEN (f)]" } else { "" };
        let title_spans = vec![
            Span::styled(
                format!(" 📖 Article Reader{mode_label} "),
                if is_focused {
                    Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.fg_dim)
                },
            ),
            Span::styled(
                " [o] Open Browser  [m] Toggle Read  [s] Star  [y] Copy URL ",
                Style::default().fg(self.theme.fg_subtle),
            ),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(Style::default().bg(self.theme.reader_bg))
            .title(Line::from(title_spans));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.height == 0 || inner_area.width == 0 {
            return;
        }

        // Fill background
        for y in inner_area.y..inner_area.y + inner_area.height {
            for x in inner_area.x..inner_area.x + inner_area.width {
                buf.set_style(Rect::new(x, y, 1, 1), Style::default().bg(self.theme.reader_bg));
            }
        }

        let article = match self.article {
            Some(a) => a,
            None => {
                let msg = "No article selected. Press 'j' / 'k' to browse articles.";
                buf.set_string(
                    inner_area.x + 3,
                    inner_area.y + 2,
                    msg,
                    Style::default().fg(self.theme.fg_dim).add_modifier(Modifier::ITALIC),
                );
                return;
            }
        };

        let formatted = render_article_to_text(article, self.theme, inner_area.width);
        let total_lines = formatted.total_lines;
        let visible_lines = inner_area.height.saturating_sub(1) as usize; // reserve 1 line for progress bar

        let start_line = self.scroll_offset.min(total_lines.saturating_sub(1));
        let end_line = (start_line + visible_lines).min(total_lines);

        for (idx, line_idx) in (start_line..end_line).enumerate() {
            let y = inner_area.y + idx as u16;
            if let Some(line) = formatted.lines.get(line_idx) {
                buf.set_line(inner_area.x + 2, y, line, inner_area.width.saturating_sub(4));
            }
        }

        // Bottom Position Indicator (e.g. Line 1/18)
        if total_lines > 0 && inner_area.height > 1 {
            let progress_y = inner_area.y + inner_area.height - 1;
            let current_pos_info = format!(" Line {}/{} ", start_line + 1, total_lines);
            let pos_span = Span::styled(
                current_pos_info.clone(),
                Style::default().fg(self.theme.fg_subtle),
            );
            let footer_line = Line::from(vec![pos_span]);
            let footer_width = unicode_width::UnicodeWidthStr::width(current_pos_info.as_str()) as u16;
            if inner_area.width > footer_width + 4 {
                let footer_x = inner_area.x + inner_area.width - footer_width - 2;
                buf.set_line(footer_x, progress_y, &footer_line, footer_width);
            }
        }
    }
}
