use crate::model::{ActivePane, ArticleSlice};
use crate::theme::Theme;
use chrono::{DateTime, Utc};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};

/// Height of one article card, in rows.
pub const CARD_HEIGHT: u16 = 3;

/// Row offset of the `k`-th visible card, given spacing measured in *half*
/// rows.
///
/// A terminal cell cannot be split, so half-row spacing is achieved by
/// distributing the gaps: at 1 half-row, every second card is preceded by a
/// blank line, averaging half a row per card. Integer division does the
/// distribution, which also keeps the gaps evenly spread at 1.5, 2.5 and so on.
pub fn card_top(k: usize, spacing_halves: u16) -> u16 {
    let k = k as u16;
    k * CARD_HEIGHT + (k * spacing_halves) / 2
}

/// Human-readable form of a half-row spacing value: "0", "0.5", "1", ...
pub fn spacing_label(spacing_halves: u16) -> String {
    if spacing_halves % 2 == 0 {
        format!("{}", spacing_halves / 2)
    } else {
        format!("{}.5", spacing_halves / 2)
    }
}

/// The card occupying `row_offset` rows below the first visible card.
pub fn card_at_row(row_offset: u16, spacing_halves: u16) -> usize {
    let mut k = 0usize;
    while card_top(k + 1, spacing_halves) <= row_offset {
        k += 1;
    }
    k
}

/// How many cards fit entirely within `avail` rows.
pub fn cards_fitting(avail: u16, spacing_halves: u16) -> usize {
    let mut k = 0usize;
    while card_top(k, spacing_halves) + CARD_HEIGHT <= avail {
        k += 1;
    }
    k
}

pub struct ArticleListView<'a> {
    pub articles: ArticleSlice<'a>,
    pub selected_index: usize,
    pub active_pane: ActivePane,
    pub theme: &'a Theme,
    pub header_title: &'a str,
    pub unread_count: usize,
    pub search_query: &'a str,
    pub is_searching: bool,
    pub scroll_offset: usize,
    pub show_icons: bool,
    pub padding: u16,
    /// Gap between cards in *half* rows: 0 = flush, 1 = half a row on average,
    /// 2 = a full row.
    pub spacing: u16,
}

impl<'a> Widget for ArticleListView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_focused = self.active_pane == ActivePane::ArticleList;
        let border_style = if is_focused {
            self.theme.active_border_style()
        } else {
            self.theme.inactive_border_style()
        };

        let count_str = if self.unread_count > 0 {
            format!(" ({})", self.unread_count)
        } else {
            String::new()
        };
        let pane_title = if self.show_icons {
            format!(" 📰 {}{count_str} ", self.header_title)
        } else {
            format!(" {}{count_str} ", self.header_title)
        };

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

        let pad = self.padding.min(inner_area.width.saturating_sub(1) / 2);
        let text_x = inner_area.x + pad;
        let text_width = inner_area.width.saturating_sub(pad * 2);

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

                buf.set_style(
                    Rect::new(inner_area.x, current_y, inner_area.width, 1),
                    search_style,
                );

                let prefix = if self.show_icons { " 🔍 " } else { " / " };
                let search_text = format!(
                    "{prefix}{}{}",
                    self.search_query,
                    if self.is_searching { "█" } else { "" }
                );
                buf.set_string(inner_area.x, current_y, &search_text, search_style);
                current_y += 1;

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
                    "No matching articles."
                } else {
                    "No articles in this feed."
                };
                buf.set_string(
                    text_x,
                    current_y + 1,
                    empty_msg,
                    Style::default().fg(self.theme.fg_dim).add_modifier(Modifier::ITALIC),
                );
            }
            return;
        }

        let avail = max_y.saturating_sub(current_y);
        // Cards that fit *entirely*; the selection is kept within these so it is
        // never the one that gets clipped.
        let visible_cards = cards_fitting(avail, self.spacing);
        let mut start_idx = self.scroll_offset;
        if self.selected_index < start_idx {
            start_idx = self.selected_index;
        } else if visible_cards > 0 && self.selected_index >= start_idx + visible_cards {
            start_idx = self.selected_index + 1 - visible_cards;
        }
        let end_idx = (start_idx + visible_cards + 1).min(self.articles.len());

        // One clock read for the whole pane rather than one per card.
        let now = Utc::now();

        for idx in start_idx..end_idx {
            let y = current_y + card_top(idx - start_idx, self.spacing);
            if y >= max_y {
                break;
            }

            let Some(article) = self.articles.get(idx) else {
                break;
            };

            // Rows of this card that actually fit. The last card is drawn
            // clipped rather than skipped, so the list runs to the bottom edge
            // instead of leaving a ragged gap above it.
            let rows = (max_y - y).min(CARD_HEIGHT);
            let is_selected = idx == self.selected_index;

            let card_bg_style = if is_selected {
                if is_focused {
                    Style::default().bg(self.theme.article_card_selected_bg)
                } else {
                    Style::default().bg(self.theme.accent_subtle)
                }
            } else {
                Style::default().bg(self.theme.article_card_bg)
            };

            // Whole card background in one call rather than width × 3 calls.
            buf.set_style(Rect::new(inner_area.x, y, inner_area.width, rows), card_bg_style);

            // Line 1: [Unread] [Star] [Title]
            let unread_marker = match (article.read, self.show_icons) {
                (false, true) => "● ",
                (false, false) => "* ",
                (true, _) => "  ",
            };
            let star_marker = match (article.starred, self.show_icons) {
                (true, true) => "★ ",
                (true, false) => "+ ",
                (false, _) => "",
            };

            let prefix_len = 2 + if article.starred { 2 } else { 0 };
            let title_avail_width = (text_width as usize).saturating_sub(prefix_len);
            let title_style = if is_selected && is_focused {
                Style::default()
                    .fg(self.theme.selection_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                self.theme.title_style(!article.read)
            };

            let line1 = Line::from(vec![
                Span::styled(unread_marker, Style::default().fg(self.theme.article_unread_dot)),
                Span::styled(
                    star_marker,
                    Style::default().fg(self.theme.article_star).add_modifier(Modifier::BOLD),
                ),
                Span::styled(truncate_string(&article.title, title_avail_width), title_style),
            ]);
            buf.set_line(text_x, y, &line1, text_width);

            // Line 2: Snippet summary
            if rows > 1 {
                let summary_text = article.summary.as_deref().unwrap_or("");
                let clean_summary =
                    truncate_string(summary_text, (text_width as usize).saturating_sub(4));
                let line2 = Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        clean_summary,
                        Style::default().fg(self.theme.article_summary_fg),
                    ),
                ]);
                buf.set_line(text_x, y + 1, &line2, text_width);
            }

            // Line 3: Feed Name (accent) + Relative Timestamp (aligned right)
            let feed_tag = &article.feed_title;
            if rows > 2 {
                let time_str =
                    format_relative_time(article.published.as_ref().unwrap_or(&article.created_at), now);
                let feed_span = Span::styled(
                    format!("   {feed_tag}"),
                    Style::default().fg(self.theme.article_meta_fg).add_modifier(Modifier::BOLD),
                );
                let feed_width = unicode_width::UnicodeWidthStr::width(feed_tag.as_str()) + 3;
                let time_width = time_str.len() + 1;

                buf.set_span(text_x, y + 2, &feed_span, text_width);

                if (text_width as usize) > feed_width + time_width + 1 {
                    let time_span =
                        Span::styled(time_str.as_str(), Style::default().fg(self.theme.fg_subtle));
                    let time_x = text_x + text_width - (time_str.len() as u16);
                    buf.set_span(time_x, y + 2, &time_span, time_str.len() as u16);
                }
            }

        }
    }
}

fn format_relative_time(date: &DateTime<Utc>, now: DateTime<Utc>) -> String {
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
