use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;

use ratatui::widgets::{Block, Borders, Clear, Widget};

pub struct ModalHelper;

impl ModalHelper {
    pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_width = (r.width * percent_x / 100).max(40).min(r.width.saturating_sub(4));
        let popup_height = (r.height * percent_y / 100).max(12).min(r.height.saturating_sub(2));
        let x = r.x + (r.width.saturating_sub(popup_width)) / 2;
        let y = r.y + (r.height.saturating_sub(popup_height)) / 2;
        Rect::new(x, y, popup_width, popup_height)
    }
}

pub struct AddFeedModal<'a> {
    pub url_input: &'a str,
    pub folder_input: &'a str,
    pub is_opml_mode: bool,
    pub focused_field: usize, // 0 = URL/Path, 1 = Folder
    pub error_msg: Option<&'a str>,
    pub is_loading: bool,
    pub theme: &'a Theme,
}

impl<'a> Widget for AddFeedModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = ModalHelper::centered_rect(60, 45, area);
        Clear.render(popup_area, buf);

        let title = if self.is_opml_mode {
            " 📥 Import Subscriptions (OPML) "
        } else {
            " ➕ Add RSS Feed / Website "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.modal_border).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(self.theme.modal_bg))
            .title(Span::styled(
                title,
                Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        if inner.height < 6 {
            return;
        }

        // Mode switch hint
        let mode_hint = if self.is_opml_mode {
            "Press [Tab] to switch field, [Ctrl+T] for Single Feed mode, [Esc] to cancel"
        } else {
            "Press [Tab] to switch field, [Ctrl+T] for OPML Import mode, [Esc] to cancel"
        };
        buf.set_string(
            inner.x + 2,
            inner.y + 1,
            mode_hint,
            Style::default().fg(self.theme.fg_dim),
        );

        let mut y = inner.y + 3;

        // Field 1: URL / OPML path
        let label1 = if self.is_opml_mode { "OPML File Path:" } else { "Feed or Website URL:" };
        buf.set_string(inner.x + 2, y, label1, Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD));
        y += 1;

        let field1_style = if self.focused_field == 0 {
            Style::default().bg(self.theme.selection_bg).fg(self.theme.selection_fg)
        } else {
            Style::default().bg(self.theme.sidebar_bg).fg(self.theme.fg)
        };

        let field1_width = inner.width.saturating_sub(4);
        for x in 0..field1_width {
            buf.set_style(Rect::new(inner.x + 2 + x, y, 1, 1), field1_style);
        }
        let input1_text = format!(" {}{}", self.url_input, if self.focused_field == 0 { "█" } else { "" });
        buf.set_string(inner.x + 2, y, &input1_text, field1_style);
        y += 2;

        // Field 2: Folder Name (Only in single feed mode)
        if !self.is_opml_mode && y + 2 < inner.y + inner.height {
            buf.set_string(inner.x + 2, y, "Folder / Category (Optional):", Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD));
            y += 1;

            let field2_style = if self.focused_field == 1 {
                Style::default().bg(self.theme.selection_bg).fg(self.theme.selection_fg)
            } else {
                Style::default().bg(self.theme.sidebar_bg).fg(self.theme.fg)
            };

            for x in 0..field1_width {
                buf.set_style(Rect::new(inner.x + 2 + x, y, 1, 1), field2_style);
            }
            let input2_text = format!(" {}{}", self.folder_input, if self.focused_field == 1 { "█" } else { "" });
            buf.set_string(inner.x + 2, y, &input2_text, field2_style);
            y += 2;
        }

        // Error or Loading message
        if let Some(err) = self.error_msg {
            buf.set_string(
                inner.x + 2,
                y,
                format!("❌ {err}"),
                Style::default().fg(self.theme.error_fg).add_modifier(Modifier::BOLD),
            );
        } else if self.is_loading {
            buf.set_string(
                inner.x + 2,
                y,
                "⏳ Fetching and verifying feed content...",
                Style::default().fg(self.theme.warning_fg),
            );
        }

        // Action buttons at bottom
        let bottom_y = inner.y + inner.height.saturating_sub(1);
        let button_text = " [Enter] Submit / Import    [Esc] Cancel ";
        buf.set_string(
            inner.x + 2,
            bottom_y,
            button_text,
            Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
        );
    }
}

pub struct ExportOpmlModal<'a> {
    pub file_path_input: &'a str,
    pub status_msg: Option<&'a str>,
    pub theme: &'a Theme,
}

impl<'a> Widget for ExportOpmlModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = ModalHelper::centered_rect(55, 30, area);
        Clear.render(popup_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.modal_border).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(self.theme.modal_bg))
            .title(Span::styled(
                " 📤 Export Feeds to OPML ",
                Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        buf.set_string(
            inner.x + 2,
            inner.y + 1,
            "Enter destination OPML file path:",
            Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD),
        );

        let field_style = Style::default().bg(self.theme.selection_bg).fg(self.theme.selection_fg);
        let width = inner.width.saturating_sub(4);
        for x in 0..width {
            buf.set_style(Rect::new(inner.x + 2 + x, inner.y + 2, 1, 1), field_style);
        }
        let text = format!(" {}█", self.file_path_input);
        buf.set_string(inner.x + 2, inner.y + 2, &text, field_style);

        if let Some(msg) = self.status_msg {
            buf.set_string(inner.x + 2, inner.y + 4, msg, Style::default().fg(self.theme.success_fg));
        }

        let bottom_y = inner.y + inner.height.saturating_sub(1);
        buf.set_string(
            inner.x + 2,
            bottom_y,
            " [Enter] Export    [Esc] Cancel ",
            Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
        );
    }
}

pub struct ThemePickerModal<'a> {
    pub themes: &'a [Theme],
    pub selected_index: usize,
    pub current_theme_name: &'a str,
    pub theme: &'a Theme,
}

impl<'a> Widget for ThemePickerModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = ModalHelper::centered_rect(50, 50, area);
        Clear.render(popup_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.modal_border).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(self.theme.modal_bg))
            .title(Span::styled(
                " 🎨 Select Theme ",
                Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height.saturating_sub(2);

        for (idx, th) in self.themes.iter().enumerate() {
            if y >= max_y {
                break;
            }

            let is_selected = idx == self.selected_index;
            let is_active = th.config.name == self.current_theme_name;

            let row_style = if is_selected {
                Style::default().bg(self.theme.selection_bg).fg(self.theme.selection_fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(self.theme.modal_bg).fg(self.theme.fg)
            };

            for x in 0..inner.width {
                buf.set_style(Rect::new(inner.x + x, y, 1, 1), row_style);
            }

            let active_marker = if is_active { " ✔ " } else { "   " };
            let cursor = if is_selected { " ▸ " } else { "   " };
            let name_text = format!("{cursor}{active_marker}{}", th.config.name);
            buf.set_string(inner.x + 1, y, &name_text, row_style);

            // Show description on right if space allows
            let desc_width = (inner.width as usize).saturating_sub(name_text.len() + 6);
            if desc_width > 10 {
                let desc_x = inner.x + inner.width - (desc_width as u16) - 1;
                let desc_style = Style::default().fg(self.theme.fg_dim);
                buf.set_string(desc_x, y, &th.config.description, desc_style);
            }

            y += 1;
        }

        let bottom_y = inner.y + inner.height.saturating_sub(1);
        buf.set_string(
            inner.x + 2,
            bottom_y,
            " [↑/↓/j/k] Browse    [Enter] Apply    [Esc] Close ",
            Style::default().fg(self.theme.accent),
        );
    }
}

pub struct HelpModal<'a> {
    pub theme: &'a Theme,
}

impl<'a> Widget for HelpModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = ModalHelper::centered_rect(75, 75, area);
        Clear.render(popup_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.modal_border).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(self.theme.modal_bg))
            .title(Span::styled(
                " ⌨️  RataRSS Keyboard Reference & Cheatsheet ",
                Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let sections = vec![
            ("Navigation & Focus", vec![
                ("Tab / Shift+Tab", "Cycle focus between Sidebar, Article List, and Reader"),
                ("h / l or Left / Right", "Move focus left / right across panes"),
                ("1 / 2 / 3", "Direct jump to Sidebar (1), Articles (2), Reader (3)"),
                ("j / k or Down / Up", "Navigate items in list or scroll reader"),
                ("Space", "Page down reader article / advance to next unread"),
                ("g / G", "Jump to top / bottom of active pane"),
            ]),
            ("Article Actions", vec![
                ("m", "Toggle Read / Unread status for selected article"),
                ("s", "Toggle Star / Bookmark for selected article"),
                ("Shift+M", "Mark all articles in current view as read"),
                ("o / Enter", "Open article URL in system default browser"),
                ("y", "Copy article URL to system clipboard"),
                ("/ or Ctrl+F", "Live search / filter articles in current view"),
            ]),
            ("Layout & Resizing", vec![
                ("< / >", "Decrease / Increase Sidebar width"),
                ("[ / ]", "Decrease / Increase Article List width"),
                ("+ / -", "Decrease / Increase Reader width"),
                ("= / Ctrl+R", "Reset pane widths to default ratios"),
                ("f or z", "Toggle Fullscreen / Zen mode for active pane"),
            ]),
            ("Feed Management & Themes", vec![
                ("a", "Add new RSS / Atom feed URL or Import OPML"),
                ("e", "Export all subscriptions to OPML format"),
                ("r", "Refresh selected feed (async background update)"),
                ("R", "Refresh all feeds"),
                ("d", "Delete selected feed or folder"),
                ("t or T", "Open interactive Theme Picker modal"),
                ("? / F1", "Toggle this Help modal"),
                ("q / Ctrl+C", "Quit RataRSS"),
            ]),
        ];

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height.saturating_sub(2);

        for (section_title, items) in sections {
            if y >= max_y {
                break;
            }

            buf.set_string(
                inner.x + 2,
                y,
                format!("── {section_title} ──"),
                Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
            );
            y += 1;

            for (key, desc) in items {
                if y >= max_y {
                    break;
                }

                let key_str = format!("  {key:<22}");
                buf.set_string(
                    inner.x + 2,
                    y,
                    &key_str,
                    Style::default().fg(self.theme.reader_h1).add_modifier(Modifier::BOLD),
                );

                buf.set_string(
                    inner.x + 26,
                    y,
                    desc,
                    Style::default().fg(self.theme.fg_dim),
                );
                y += 1;
            }
            y += 1;
        }

        let bottom_y = inner.y + inner.height.saturating_sub(1);
        buf.set_string(
            inner.x + 2,
            bottom_y,
            " Press [Esc] or [?] to close ",
            Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
        );
    }
}

pub struct ConfirmDeleteModal<'a> {
    pub target_name: &'a str,
    pub is_folder: bool,
    pub theme: &'a Theme,
}

impl<'a> Widget for ConfirmDeleteModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = ModalHelper::centered_rect(50, 25, area);
        Clear.render(popup_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.error_fg).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(self.theme.modal_bg))
            .title(Span::styled(
                " ⚠️ Confirm Deletion ",
                Style::default().fg(self.theme.error_fg).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let item_type = if self.is_folder { "folder and all its feeds" } else { "feed" };
        let msg = format!("Are you sure you want to delete the {item_type}:");
        buf.set_string(inner.x + 2, inner.y + 1, &msg, Style::default().fg(self.theme.fg));
        buf.set_string(
            inner.x + 4,
            inner.y + 2,
            format!("\"{}\"", self.target_name),
            Style::default().fg(self.theme.error_fg).add_modifier(Modifier::BOLD),
        );

        let bottom_y = inner.y + inner.height.saturating_sub(1);
        buf.set_string(
            inner.x + 2,
            bottom_y,
            " [y / Enter] Confirm Delete    [n / Esc] Cancel ",
            Style::default().fg(self.theme.fg).add_modifier(Modifier::BOLD),
        );
    }
}
