use crate::model::{ActivePane, Article};
use crate::theme::Theme;
use chrono::{DateTime, Utc};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};

pub struct ArticleListView<'a> {
    pub articles: &'a [Article],
    pub selected_index: usize,
    pub active_pane: ActivePane,
    pub theme: &'a Theme,
    pub header_title: &'a str,
    pub unread_count: usize,
    pub search_query: &'a str,
    pub is_searching: bool,
    pub scroll_offset: usize,
}

impl<'a> Widget for ArticleListView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_focused = self.active_pane == ActivePane::ArticleList;
        let border_style = if is_focused {
            self.theme.active_border_style()
        } else {
            self.theme.inactive_border_style()
        };

        // Title with unread count
        let count_str = if self.unread_count > 0 {
            format!(" ({} unread)", self.unread_count)
        } else {
            String::new()
        };
        let pane_title = format!(" 📰 {}{count_str} ", self.header_title);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(Style::default().bg(self.theme.article_list_bg))
            .title(Span::styled(
                pane_title,
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

        let mut current_y = inner_area.y;
        let max_y = inner_area.y + inner_area.height;

        // Search Bar (if searching or search query is not empty)
        if self.is_searching || !self.search_query.is_empty() {
            if current_y < max_y {
                let search_style = if self.is_searching {
                    Style::default().bg(self.theme.selection_bg).fg(self.theme.selection_fg)
                } else {
                    Style::default().bg(self.theme.sidebar_bg).fg(self.theme.fg_dim)
                };

                for x in inner_area.x..inner_area.x + inner_area.width {
                    buf.set_style(Rect::new(x, current_y, 1, 1), search_style);
                }

                let search_text = format!(" 🔍 Search: {}{}", self.search_query, if self.is_searching { "█" } else { "" });
                buf.set_string(inner_area.x, current_y, &search_text, search_style);
                current_y += 1;

                // Subtle separator under search bar
                if current_y < max_y {
                    let sep = "─".repeat(inner_area.width as usize);
                    buf.set_string(inner_area.x, current_y, &sep, Style::default().fg(self.theme.border_inactive));
                    current_y += 1;
                }
            }
        }

        if self.articles.is_empty() {
            if current_y < max_y {
                let empty_msg = if !self.search_query.is_empty() {
                    "No matching articles found."
                } else {
                    "No articles in this feed."
                };
                buf.set_string(
                    inner_area.x + 2,
                    current_y + 1,
                    empty_msg,
                    Style::default().fg(self.theme.fg_dim).add_modifier(Modifier::ITALIC),
                );
            }
            return;
        }

        // Each card takes 3 lines + 1 optional divider line (or 3 lines compact)
        let card_height = 3;
        let visible_cards = ((max_y.saturating_sub(current_y)) / card_height as u16) as usize;
        let start_idx = self.scroll_offset;
        let end_idx = (start_idx + visible_cards + 1).min(self.articles.len());

        let mut y = current_y;

        for idx in start_idx..end_idx {
            if y + (card_height as u16) > max_y {
                break;
            }

            let article = &self.articles[idx];
            let is_selected = idx == self.selected_index;

            // Background for the 3-line card
            let card_bg_style = if is_selected {
                if is_focused {
                    Style::default().bg(self.theme.article_card_selected_bg)
                } else {
                    Style::default().bg(self.theme.accent_subtle)
                }
            } else {
                Style::default().bg(self.theme.article_card_bg)
            };

            for row_offset in 0..card_height as u16 {
                for col in 0..inner_area.width {
                    buf.set_style(Rect::new(inner_area.x + col, y + row_offset, 1, 1), card_bg_style);
                }
            }

            // Line 1: [Unread Dot / Star] [Title]
            let unread_dot = if !article.read { "● " } else { "  " };
            let star_icon = if article.starred { "★ " } else { "" };
            let dot_style = Style::default().fg(self.theme.article_unread_dot);
            let star_style = Style::default().fg(self.theme.article_star).add_modifier(Modifier::BOLD);

            let prefix_spans = vec![
                Span::styled(unread_dot, dot_style),
                Span::styled(star_icon, star_style),
            ];
            let prefix_len = if !article.read { 2 } else { 2 } + if article.starred { 2 } else { 0 };

            let title_avail_width = (inner_area.width as usize).saturating_sub(prefix_len + 1);
            let title_text = truncate_string(&article.title, title_avail_width);
            let title_style = if is_selected && is_focused {
                Style::default()
                    .fg(self.theme.selection_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                self.theme.title_style(!article.read)
            };

            let mut line1_spans = prefix_spans;
            line1_spans.push(Span::styled(title_text, title_style));
            let line1 = Line::from(line1_spans);
            buf.set_line(inner_area.x + 1, y, &line1, inner_area.width.saturating_sub(1));

            // Line 2: Snippet summary
            let summary_text = article
                .summary
                .as_deref()
                .unwrap_or("");
            let clean_summary = truncate_string(summary_text, (inner_area.width as usize).saturating_sub(5));
            let summary_style = Style::default().fg(self.theme.article_summary_fg);
            let line2 = Line::from(vec![
                Span::raw("    "),
                Span::styled(clean_summary, summary_style),
            ]);
            buf.set_line(inner_area.x + 1, y + 1, &line2, inner_area.width.saturating_sub(1));

            // Line 3: Feed Name (accent) + Relative Timestamp (aligned right)
            let feed_tag = &article.feed_title;
            let time_str = format_relative_time(article.published.as_ref().unwrap_or(&article.created_at));

            let feed_style = Style::default().fg(self.theme.article_meta_fg).add_modifier(Modifier::BOLD);
            let time_style = Style::default().fg(self.theme.fg_subtle);

            let feed_span = Span::styled(format!("    {feed_tag}"), feed_style);
            let time_span = Span::styled(format!("{time_str} "), time_style);

            let feed_width = unicode_width::UnicodeWidthStr::width(feed_tag.as_str()) + 4;
            let time_width = time_str.len() + 1;

            buf.set_span(inner_area.x + 1, y + 2, &feed_span, inner_area.width.saturating_sub(1));

            if (inner_area.width as usize) > feed_width + time_width + 2 {
                let time_x = inner_area.x + inner_area.width - (time_width as u16) - 1;
                buf.set_span(time_x, y + 2, &time_span, time_width as u16);
            }

            y += card_height as u16;
        }
    }
}

fn format_relative_time(date: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*date);

    if duration.num_seconds() < 60 {
        "Just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        date.format("%I:%M %p").to_string()
    } else if duration.num_days() == 1 {
        "Yesterday".to_string()
    } else if duration.num_days() < 7 {
        date.format("%a %I:%M %p").to_string()
    } else {
        date.format("%b %d").to_string()
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
