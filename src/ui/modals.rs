use crate::config::AppConfig;
use crate::theme::Theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Clear, Widget};

pub struct ModalHelper;

impl ModalHelper {
    pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_width = (r.width * percent_x / 100).max(52).min(r.width.saturating_sub(2));
        let popup_height = (r.height * percent_y / 100).max(14).min(r.height.saturating_sub(2));
        let x = r.x + (r.width.saturating_sub(popup_width)) / 2;
        let y = r.y + (r.height.saturating_sub(popup_height)) / 2;
        Rect::new(x, y, popup_width, popup_height)
    }

    pub fn bottom_sheet_rect(percent_w: u16, height_lines: u16, r: Rect) -> Rect {
        let width = (r.width * percent_w / 100).max(56).min(r.width.saturating_sub(2));
        let height = height_lines.min(r.height.saturating_sub(2));
        let x = r.x + (r.width.saturating_sub(width)) / 2;
        let y = r.y + r.height.saturating_sub(height + 1);
        Rect::new(x, y, width, height)
    }
}

pub struct AddFeedModal<'a> {
    pub show_icons: bool,
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
            if self.show_icons { " 📥 Import Subscriptions (OPML) " } else { " Import Subscriptions (OPML) " }
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
        buf.set_style(Rect::new(inner.x + 2, y, field1_width, 1), field1_style);
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

            buf.set_style(Rect::new(inner.x + 2, y, field1_width, 1), field2_style);
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
    pub show_icons: bool,
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
                if self.show_icons { " 📤 Export Feeds to OPML " } else { " Export Feeds to OPML " },
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
        buf.set_style(Rect::new(inner.x + 2, inner.y + 2, width, 1), field_style);
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
    pub show_icons: bool,
    pub themes: &'a [Theme],
    pub selected_index: usize,
    pub current_theme_name: &'a str,
    pub theme: &'a Theme,
}

impl<'a> Widget for ThemePickerModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = ModalHelper::centered_rect(72, 65, area);
        Clear.render(popup_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.modal_border).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(self.theme.modal_bg))
            .title(Span::styled(
                if self.show_icons {
                    format!(" 🎨 Select Theme ({} available) ", self.themes.len())
                } else {
                    format!(" Select Theme ({} available) ", self.themes.len())
                },
                Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        if inner.height < 4 || inner.width < 10 {
            return;
        }

        let visible_height = inner.height.saturating_sub(2) as usize;
        let mut scroll_offset = 0;
        if self.selected_index >= visible_height {
            scroll_offset = self.selected_index + 1 - visible_height;
        }
        if scroll_offset + visible_height > self.themes.len() {
            scroll_offset = self.themes.len().saturating_sub(visible_height);
        }

        let end_idx = (scroll_offset + visible_height).min(self.themes.len());

        let mut y = inner.y + 1;

        // Up arrow indicator if scrollable
        if scroll_offset > 0 {
            let indicator = " ▲ (more themes above)";
            buf.set_string(inner.x + 2, inner.y, indicator, Style::default().fg(self.theme.accent));
        }

        for idx in scroll_offset..end_idx {
            if let Some(th) = self.themes.get(idx) {
                let is_selected = idx == self.selected_index;
                let is_active = th.config.name == self.current_theme_name;

                let row_style = if is_selected {
                    Style::default().bg(self.theme.selection_bg).fg(self.theme.selection_fg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().bg(self.theme.modal_bg).fg(self.theme.fg)
                };

                buf.set_style(Rect::new(inner.x, y, inner.width, 1), row_style);

                let active_marker = if is_active { "✔ " } else { "  " };
                let cursor = if is_selected { "▸ " } else { "  " };
                let name_display = format!("{cursor}{active_marker}{:<22}", th.config.name);
                buf.set_string(inner.x + 1, y, &name_display, row_style);

                // Show description on right if space allows
                let left_w = 27;
                if (inner.width as usize) > left_w + 10 {
                    let desc_w = (inner.width as usize).saturating_sub(left_w + 2);
                    let desc_str = if th.config.description.chars().count() > desc_w {
                        let trunc: String = th.config.description.chars().take(desc_w.saturating_sub(1)).collect();
                        format!("{trunc}…")
                    } else {
                        th.config.description.clone()
                    };
                    let desc_x = inner.x + left_w as u16;
                    let desc_style = if is_selected {
                        Style::default().fg(self.theme.selection_fg)
                    } else {
                        Style::default().fg(self.theme.fg_dim)
                    };
                    buf.set_string(desc_x, y, &desc_str, desc_style);
                }

                y += 1;
            }
        }

        // Down arrow indicator if scrollable
        if end_idx < self.themes.len() {
            let indicator = " ▼ (more themes below)";
            buf.set_string(inner.x + 2, inner.y + inner.height.saturating_sub(1), indicator, Style::default().fg(self.theme.accent));
        }

        let bottom_y = inner.y + inner.height.saturating_sub(1);
        buf.set_string(
            inner.x + 2,
            bottom_y,
            " [↑/↓/j/k] Browse    [Enter] Apply Theme    [Esc] Close ",
            Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSubView {
    Main,
    Keybindings,
}

pub struct ConfigMenuModal<'a> {
    pub config: &'a AppConfig,
    pub selected_index: usize,
    pub subview: ConfigSubView,
    pub keybind_selected_idx: usize,
    pub theme: &'a Theme,
}

impl<'a> Widget for ConfigMenuModal<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.subview {
            ConfigSubView::Main => self.render_main_config(area, buf),
            ConfigSubView::Keybindings => self.render_keybindings(area, buf),
        }
    }
}

impl<'a> ConfigMenuModal<'a> {
    fn render_main_config(&self, area: Rect, buf: &mut Buffer) {
        let height_lines = 17;
        let popup_area = ModalHelper::bottom_sheet_rect(88, height_lines, area);
        Clear.render(popup_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.modal_border).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(self.theme.modal_bg))
            .title(Span::styled(
                if self.config.show_icons {
                    " ⚙️  Settings (/ to close) "
                } else {
                    " Settings (/ to close) "
                },
                Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        // Labels carry their icon only when icons are enabled.
        let icons = self.config.show_icons;
        let items = [
            (
                if icons { "🎨 Theme" } else { "Theme" },
                format!("◀  {}  ▶ (Enter / ← / → to change)", self.config.theme),
            ),
            (
                if icons { "🔄 Auto-Refresh on Startup" } else { "Auto-Refresh on Startup" },
                if self.config.auto_refresh_on_startup {
                    "[✓] Enabled (Space / Enter to toggle)".to_string()
                } else {
                    "[ ] Disabled (Space / Enter to toggle)".to_string()
                },
            ),
            (
                if icons { "⏱️  Refresh Interval" } else { "Refresh Interval" },
                format!("◀  {} minutes  ▶ (← / → to change)", self.config.refresh_interval_minutes),
            ),
            (
                if icons { "📖 Mark Read on Open" } else { "Mark Read on Open" },
                if self.config.mark_read_on_open {
                    "[✓] Enabled (Space / Enter to toggle)".to_string()
                } else {
                    "[ ] Disabled (Space / Enter to toggle)".to_string()
                },
            ),
            (
                if icons { "🔤 Wrap Article Text" } else { "Wrap Article Text" },
                if self.config.wrap_article_text {
                    "[✓] Enabled (Space / Enter to toggle)".to_string()
                } else {
                    "[ ] Disabled (Space / Enter to toggle)".to_string()
                },
            ),
            (
                if icons { "🖼️  Show Badges & Icons" } else { "Show Badges & Icons" },
                if self.config.show_icons {
                    "[✓] Enabled (Space / Enter to toggle)".to_string()
                } else {
                    "[ ] Disabled (Space / Enter to toggle)".to_string()
                },
            ),
            (
                if icons { "📐 Content Padding" } else { "Content Padding" },
                format!("◀  {} cell(s)  ▶ (← / → to change)", self.config.padding),
            ),
            (
                if icons { "↕️  Article Spacing" } else { "Article Spacing" },
                format!(
                    "◀  {} row(s)  ▶ (← / → in half-row steps)",
                    crate::ui::article_list::spacing_label(self.config.article_spacing)
                ),
            ),
            (
                if icons { "❔ Shortcut Hints in Status Bar" } else { "Shortcut Hints in Status Bar" },
                if self.config.show_help_hints {
                    "[✓] Shown (Space / Enter, or ?? anywhere)".to_string()
                } else {
                    "[ ] Hidden (Space / Enter, or ?? anywhere)".to_string()
                },
            ),
            (
                if icons { "📏 Layout Pane Ratios" } else { "Layout Pane Ratios" },
                format!(
                    "Sidebar: {}%  |  Articles: {}%  |  Reader: {}%  (← / → adjust, = reset)",
                    self.config.sidebar_ratio, self.config.article_list_ratio, self.config.reader_ratio
                ),
            ),
            (
                if icons { "⌨️  Custom Keybindings" } else { "Custom Keybindings" },
                "View & configure shortcuts (Press Enter to open)".to_string(),
            ),
        ];

        let mut y = inner.y + 1;
        let max_y = inner.y + inner.height.saturating_sub(2);

        for (idx, (label, value)) in items.iter().enumerate() {
            if y >= max_y {
                break;
            }

            let is_selected = idx == self.selected_index;
            let row_style = if is_selected {
                Style::default().bg(self.theme.selection_bg).fg(self.theme.selection_fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(self.theme.modal_bg).fg(self.theme.fg)
            };

            buf.set_style(Rect::new(inner.x, y, inner.width, 1), row_style);

            let cursor = if is_selected { " ▸ " } else { "   " };
            let label_text = format!("{cursor}{:<28}", label);
            buf.set_string(inner.x + 1, y, &label_text, row_style);

            let val_style = if is_selected {
                Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.theme.fg_dim)
            };
            buf.set_string(inner.x + 32, y, value, val_style);

            y += 1;
        }

        let bottom_y = inner.y + inner.height.saturating_sub(1);
        let help_text = " [↑/↓/j/k] Select   [←/→/Space/Enter] Modify   [/ or Esc] Close ";
        buf.set_string(
            inner.x + 2,
            bottom_y,
            help_text,
            Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
        );
    }

    fn render_keybindings(&self, area: Rect, buf: &mut Buffer) {
        let popup_area = ModalHelper::centered_rect(80, 70, area);
        Clear.render(popup_area, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.modal_border).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(self.theme.modal_bg))
            .title(Span::styled(
                if self.config.show_icons {
                    " ⌨️  Keybindings (saved in config.toml) "
                } else {
                    " Keybindings (saved in config.toml) "
                },
                Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let nav_keys = format!("{}, {}", self.config.keybindings.nav_down, self.config.keybindings.nav_up);
        let page_keys = format!("{}, {}", self.config.keybindings.page_down, self.config.keybindings.page_up);
        let jump_keys = format!("{}, {}", self.config.keybindings.jump_top, self.config.keybindings.jump_bottom);

        let bindings: [(&str, &str, &str); 26] = [
            ("Toggle Config Menu", self.config.keybindings.toggle_config.as_str(), "Open / close this popup"),
            ("Quit Application", self.config.keybindings.quit.as_str(), "Exit RataRSS"),
            ("Help / Cheatsheet", self.config.keybindings.help.as_str(), "Open help modal"),
            ("Select Theme", self.config.keybindings.theme_picker.as_str(), "Open theme picker dialog"),
            ("Add Feed / OPML", self.config.keybindings.add_feed.as_str(), "Subscribe to new feed"),
            ("Export OPML", self.config.keybindings.export_opml.as_str(), "Export subscriptions"),
            ("Delete Feed / Folder", self.config.keybindings.delete_item.as_str(), "Remove selected item"),
            ("Zen / Fullscreen", self.config.keybindings.toggle_zen.as_str(), "Maximize current pane"),
            ("Search Articles", self.config.keybindings.search.as_str(), "Fuzzy-find articles in this view"),
            ("Search Feeds", self.config.keybindings.search_feeds.as_str(), "Fuzzy-find a feed in the sidebar"),
            ("Refresh Feed", self.config.keybindings.refresh_current.as_str(), "Sync current feed"),
            ("Refresh All Feeds", self.config.keybindings.refresh_all.as_str(), "Sync all feeds"),
            ("Toggle Read / Unread", self.config.keybindings.toggle_read.as_str(), "Toggle article read status"),
            ("Mark All Read", self.config.keybindings.mark_all_read.as_str(), "Mark view as read"),
            ("Star / Bookmark", self.config.keybindings.toggle_star.as_str(), "Toggle article starred"),
            ("Open in Browser", self.config.keybindings.open_browser.as_str(), "Open article URL in browser"),
            ("Copy URL", self.config.keybindings.copy_url.as_str(), "Copy article URL to clipboard"),
            ("Next Pane", self.config.keybindings.focus_next_pane.as_str(), "Move focus right"),
            ("Previous Pane", self.config.keybindings.focus_prev_pane.as_str(), "Move focus left"),
            ("Jump to Feeds / Articles / Reader", "1, 2, 3", "Direct pane focus"),
            ("Navigate Down / Up", nav_keys.as_str(), "Scroll / select items"),
            ("Page Down / Up", page_keys.as_str(), "Page scroll"),
            ("Space Scroll / Advance", self.config.keybindings.space_advance.as_str(), "Scroll reader / advance"),
            ("Jump Top / Bottom", jump_keys.as_str(), "Jump to extremes"),
            ("Resize Panes", "< / > / [ / ] / + / -", "Adjust layout ratios"),
            ("Reset Layout", self.config.keybindings.reset_layout.as_str(), "Reset ratios to defaults"),
        ];

        let visible_height = inner.height.saturating_sub(3) as usize;
        let mut scroll_offset = 0;
        if self.keybind_selected_idx >= visible_height {
            scroll_offset = self.keybind_selected_idx + 1 - visible_height;
        }
        if scroll_offset + visible_height > bindings.len() {
            scroll_offset = bindings.len().saturating_sub(visible_height);
        }

        let end_idx = (scroll_offset + visible_height).min(bindings.len());

        let mut y = inner.y + 1;

        // Header line
        let header_style = Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD);
        buf.set_string(inner.x + 3, y, "Action", header_style);
        buf.set_string(inner.x + 34, y, "Key Binding", header_style);
        buf.set_string(inner.x + 58, y, "Description", header_style);
        y += 1;

        for idx in scroll_offset..end_idx {
            let (action, keys, desc) = &bindings[idx];
            let is_selected = idx == self.keybind_selected_idx;

            let row_style = if is_selected {
                Style::default().bg(self.theme.selection_bg).fg(self.theme.selection_fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(self.theme.modal_bg).fg(self.theme.fg)
            };

            buf.set_style(Rect::new(inner.x, y, inner.width, 1), row_style);

            let cursor = if is_selected { "▸ " } else { "  " };
            buf.set_string(inner.x + 1, y, &format!("{cursor}{:<30}", action), row_style);
            buf.set_string(inner.x + 34, y, &format!("{:<22}", keys), if is_selected { row_style } else { Style::default().fg(self.theme.reader_h1) });
            buf.set_string(inner.x + 58, y, desc, if is_selected { row_style } else { Style::default().fg(self.theme.fg_dim) });

            y += 1;
        }

        let bottom_y = inner.y + inner.height.saturating_sub(1);
        buf.set_string(
            inner.x + 2,
            bottom_y,
            " [↑/↓/j/k] Browse    [r] Reset to Defaults    [Esc / Backspace] Back to Menu ",
            Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
        );
    }
}

pub struct HelpModal<'a> {
    pub theme: &'a Theme,
    pub show_icons: bool,
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
                if self.show_icons {
                    " ⌨️  RataRSS Shortcuts "
                } else {
                    " RataRSS Shortcuts "
                },
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
                ("Ctrl+A", "Fuzzy search articles in the current view"),
                ("Ctrl+F", "Fuzzy search feeds in the sidebar"),
            ]),
            ("Configuration & Themes", vec![
                ("/", "Toggle interactive Configuration Menu popup"),
                ("t or T", "Open interactive Theme Picker modal (27 themes)"),
                ("a", "Add new RSS / Atom feed URL or Import OPML"),
                ("e", "Export all subscriptions to OPML format"),
                ("r / R", "Refresh selected feed / Refresh all feeds"),
                ("d", "Delete selected feed or folder"),
                ("f or z", "Toggle Fullscreen / Zen mode for active pane"),
                ("? / F1", "Open this reference"),
                ("??", "Toggle the compact shortcut hints in the status bar"),
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

                let key_str = format!("  {key:<24}");
                buf.set_string(
                    inner.x + 2,
                    y,
                    &key_str,
                    Style::default().fg(self.theme.reader_h1).add_modifier(Modifier::BOLD),
                );

                buf.set_string(
                    inner.x + 28,
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
            " [Esc] close   [?] close and toggle status-bar hints ",
            Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD),
        );
    }
}

pub struct ConfirmDeleteModal<'a> {
    pub show_icons: bool,
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
                if self.show_icons { " ⚠️ Confirm Deletion " } else { " Confirm Deletion " },
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
