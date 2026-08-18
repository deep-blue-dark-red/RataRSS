use crate::model::{ActivePane, SidebarItem};
use crate::theme::Theme;
use crate::ui::widgets::Badge;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};

pub struct SidebarView<'a> {
    pub items: &'a [SidebarItem],
    pub selected_index: usize,
    pub active_pane: ActivePane,
    pub theme: &'a Theme,
    pub scroll_offset: usize,
    pub show_icons: bool,
    pub padding: u16,
}

impl<'a> Widget for SidebarView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_focused = self.active_pane == ActivePane::Sidebar;
        let border_style = if is_focused {
            self.theme.active_border_style()
        } else {
            self.theme.inactive_border_style()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(Style::default().bg(self.theme.sidebar_bg))
            .title(Span::styled(
                if self.show_icons { " 🗂 Feeds " } else { " Feeds " },
                if is_focused {
                    Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.sidebar_header_fg)
                },
            ));

        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.height == 0 || inner_area.width == 0 {
            return;
        }

        // Content is inset by the configured padding; row backgrounds still
        // span the full pane so selection highlights stay flush.
        let pad = self.padding.min(inner_area.width.saturating_sub(1) / 2);
        let text_x = inner_area.x + pad;
        let text_width = inner_area.width.saturating_sub(pad * 2);

        let visible_height = inner_area.height as usize;
        let mut start_idx = self.scroll_offset;
        if self.selected_index < start_idx {
            start_idx = self.selected_index;
        } else if visible_height > 0 && self.selected_index >= start_idx + visible_height {
            start_idx = self.selected_index + 1 - visible_height;
        }
        let end_idx = (start_idx + visible_height).min(self.items.len());

        for (row_offset, idx) in (start_idx..end_idx).enumerate() {
            let y = inner_area.y + row_offset as u16;
            let item = &self.items[idx];
            let is_selected = idx == self.selected_index;

            // Row background style
            let row_bg_style = if is_selected {
                if is_focused {
                    Style::default()
                        .bg(self.theme.selection_bg)
                        .fg(self.theme.selection_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .bg(self.theme.accent_subtle)
                        .fg(self.theme.fg)
                }
            } else {
                Style::default().bg(self.theme.sidebar_bg).fg(self.theme.fg)
            };

            // Fill the row in one call rather than one per cell.
            buf.set_style(Rect::new(inner_area.x, y, inner_area.width, 1), row_bg_style);

            match item {
                SidebarItem::SmartHeader => {
                    let text = " SMART FEEDS";
                    let line = Line::from(vec![Span::styled(
                        text,
                        Style::default()
                            .fg(self.theme.sidebar_header_fg)
                            .add_modifier(Modifier::BOLD),
                    )]);
                    buf.set_line(text_x, y, &line, text_width);
                }
                SidebarItem::Smart(kind, count) => {
                    let title = kind.title();
                    let badge_text = if *count > 0 {
                        format!("{count}")
                    } else {
                        String::new()
                    };

                    let prefix = if is_selected { " ▸ " } else { "   " };
                    let left_text = if self.show_icons {
                        format!("{prefix}{}  {title}", kind.icon())
                    } else {
                        format!("{prefix}{title}")
                    };
                    let left_len = unicode_width::UnicodeWidthStr::width(left_text.as_str());

                    let line_left = Span::styled(
                        left_text,
                        if is_selected {
                            Style::default().fg(self.theme.selection_fg).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(self.theme.fg)
                        },
                    );

                    buf.set_span(text_x, y, &line_left, text_width);

                    if !badge_text.is_empty() {
                        let badge = Badge::new(&badge_text, self.theme, is_selected);
                        let badge_span = badge.to_span();
                        let badge_width = badge_text.len() + 2;
                        if (text_width as usize) > left_len + badge_width + 1 {
                            let badge_x = text_x + text_width - badge_width as u16;
                            buf.set_span(badge_x, y, &badge_span, badge_width as u16);
                        }
                    }
                }
                SidebarItem::FolderHeader {
                    name,
                    is_expanded,
                    unread_count,
                    feed_count: _,
                } => {
                    let arrow = if *is_expanded { "▾" } else { "▸" };
                    let prefix = if is_selected { " ▸ " } else { "   " };
                    let folder_title = if self.show_icons {
                        let icon = if *is_expanded { "📂" } else { "📁" };
                        format!("{prefix}{arrow} {icon} {name}")
                    } else {
                        format!("{prefix}{arrow} {name}")
                    };
                    let left_len = unicode_width::UnicodeWidthStr::width(folder_title.as_str());

                    let span = Span::styled(
                        folder_title,
                        if is_selected {
                            Style::default()
                                .fg(self.theme.sidebar_folder_fg)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                                .fg(self.theme.sidebar_folder_fg)
                                .add_modifier(Modifier::BOLD)
                        },
                    );
                    buf.set_span(text_x, y, &span, text_width);

                    let badge_text = if *unread_count > 0 {
                        format!("{unread_count}")
                    } else {
                        String::new()
                    };

                    if !badge_text.is_empty() {
                        let badge = Badge::new(&badge_text, self.theme, is_selected);
                        let badge_span = badge.to_span();
                        let badge_width = badge_text.len() + 2;
                        if (text_width as usize) > left_len + badge_width + 1 {
                            let badge_x = text_x + text_width - badge_width as u16;
                            buf.set_span(badge_x, y, &badge_span, badge_width as u16);
                        }
                    }
                }
                SidebarItem::Feed {
                    feed_id: _,
                    title,
                    folder,
                    unread_count,
                    has_error,
                } => {
                    let indent = if folder.is_some() { "      " } else { "   " };
                    let prefix = if is_selected { "▸" } else { " " };
                    let status_icon = if *has_error {
                        if self.show_icons { "⚠️" } else { "!" }
                    } else if *unread_count > 0 {
                        "•"
                    } else {
                        " "
                    };

                    let title_avail_width = text_width.saturating_sub(12) as usize;
                    let display_title = truncate_string(title, title_avail_width);

                    let feed_text = format!("{prefix}{indent}{status_icon} {display_title}");
                    let left_len = unicode_width::UnicodeWidthStr::width(feed_text.as_str());

                    let span = Span::styled(
                        feed_text,
                        if is_selected {
                            Style::default()
                                .fg(self.theme.sidebar_feed_fg)
                                .add_modifier(Modifier::BOLD)
                        } else if *unread_count > 0 {
                            Style::default()
                                .fg(self.theme.sidebar_feed_fg)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(self.theme.fg_dim)
                        },
                    );
                    buf.set_span(text_x, y, &span, text_width);

                    if *unread_count > 0 {
                        let badge_text = format!("{unread_count}");
                        let badge = Badge::new(&badge_text, self.theme, is_selected);
                        let badge_span = badge.to_span();
                        let badge_width = badge_text.len() + 2;
                        if (text_width as usize) > left_len + badge_width + 1 {
                            let badge_x = text_x + text_width - badge_width as u16;
                            buf.set_span(badge_x, y, &badge_span, badge_width as u16);
                        }
                    }
                }
            }
        }
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let prefix: String = s.chars().take(max_len.saturating_sub(1)).collect();
        format!("{prefix}…")
    }
}
