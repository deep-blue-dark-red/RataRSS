pub mod article_list;
pub mod modals;
pub mod reader_view;
pub mod sidebar;
pub mod widgets;

use crate::app::App;
use crate::model::ActivePane;
use crate::ui::article_list::ArticleListView;
use crate::ui::modals::{
    AddFeedModal, ConfigMenuModal, ConfirmDeleteModal, ExportOpmlModal, HelpModal, ThemePickerModal,
};
use crate::ui::reader_view::ReaderView;
use crate::ui::sidebar::SidebarView;
use crate::ui::widgets::get_spinner_frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::Frame;

/// Compact shortcut hints shown in the status bar when enabled (`??`).
const HELP_HINTS: &str = "? help · / config · T theme · a add · r sync · f zen · q quit";

pub fn render_app(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let theme = &app.theme;

    // The old top header bar is gone: the brand, sync state and shortcut hints
    // all live in the single status line now, so the panes get that row back.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // 3-Pane Body
            Constraint::Length(1), // Bottom Status Bar
        ])
        .split(area);

    let body_area = chunks[0];
    let status_bar_area = chunks[1];

    // --- Render Main Body ---
    if app.is_zen_mode {
        // Fullscreen active pane (usually reader or articles)
        match app.active_pane {
            ActivePane::Sidebar => {
                frame.render_widget(sidebar_view(app), body_area);
            }
            ActivePane::ArticleList => {
                frame.render_widget(article_list_view(app), body_area);
            }
            ActivePane::Reader => {
                render_reader(app, body_area, true, frame);
            }
        }
    } else {
        // 3-Pane Horizontal Split
        let pane_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(app.sidebar_width_percent),
                Constraint::Percentage(app.article_width_percent),
                Constraint::Percentage(app.reader_width_percent),
            ])
            .split(body_area);

        frame.render_widget(sidebar_view(app), pane_chunks[0]);
        frame.render_widget(article_list_view(app), pane_chunks[1]);
        render_reader(app, pane_chunks[2], false, frame);
    }

    // --- Render Bottom Status Bar ---
    render_status_bar(app, status_bar_area, frame.buffer_mut());

    // --- Render Active Modals / Overlays ---
    if app.show_config_modal {
        let modal = ConfigMenuModal {
            config: &app.config,
            selected_index: app.config_menu_selected_idx,
            subview: app.config_menu_subview,
            keybind_selected_idx: app.config_keybind_selected_idx,
            theme,
        };
        frame.render_widget(modal, area);
    } else if app.show_add_modal {
        let modal = AddFeedModal {
            show_icons: app.config.show_icons,
            url_input: &app.modal_url_input,
            folder_input: &app.modal_folder_input,
            is_opml_mode: app.modal_is_opml_mode,
            focused_field: app.modal_focused_field,
            error_msg: app.modal_error.as_deref(),
            is_loading: app.modal_is_loading,
            theme,
        };
        frame.render_widget(modal, area);
    } else if app.show_export_modal {
        let modal = ExportOpmlModal {
            show_icons: app.config.show_icons,
            file_path_input: &app.modal_export_path,
            status_msg: app.modal_export_status.as_deref(),
            theme,
        };
        frame.render_widget(modal, area);
    } else if app.show_theme_modal {
        let modal = ThemePickerModal {
            show_icons: app.config.show_icons,
            themes: &app.all_themes,
            selected_index: app.theme_picker_selected_idx,
            current_theme_name: &app.theme.config.name,
            theme,
        };
        frame.render_widget(modal, area);
    } else if app.show_help_modal {
        let modal = HelpModal {
            theme,
            show_icons: app.config.show_icons,
        };
        frame.render_widget(modal, area);
    } else if app.show_delete_modal {
        let (name, is_folder) = app.get_delete_target_info();
        let modal = ConfirmDeleteModal {
            show_icons: app.config.show_icons,
            target_name: &name,
            is_folder,
            theme,
        };
        frame.render_widget(modal, area);
    }
}

fn sidebar_view(app: &App) -> SidebarView<'_> {
    SidebarView {
        items: &app.sidebar_items,
        selected_index: app.sidebar_selected_idx,
        active_pane: app.active_pane,
        theme: &app.theme,
        scroll_offset: app.sidebar_scroll_offset,
        show_icons: app.config.show_icons,
        padding: app.config.padding,
        search_query: &app.feed_search_query,
        is_searching: app.is_searching_feeds,
    }
}

fn article_list_view(app: &App) -> ArticleListView<'_> {
    ArticleListView {
        // A borrowed view of the article list; nothing is cloned per frame.
        articles: app.visible_articles(),
        selected_index: app.article_selected_idx,
        active_pane: app.active_pane,
        theme: &app.theme,
        header_title: &app.current_view_title,
        unread_count: app.current_view_unread_count,
        search_query: &app.search_query,
        is_searching: app.is_searching,
        scroll_offset: app.article_scroll_offset,
        show_icons: app.config.show_icons,
        padding: app.config.padding,
        spacing: app.config.article_spacing,
    }
}

/// The reader needs its formatted text, which is cached in the app and only
/// re-rendered when the article, pane width or theme changes.
fn render_reader(app: &App, area: Rect, is_zen_mode: bool, frame: &mut Frame) {
    let inner_width = ReaderView::inner_width(area, app.config.padding);
    // Remember where the text landed so clicks can be mapped back onto links.
    app.set_reader_text_area(ReaderView::text_area(area, app.config.padding));
    app.with_formatted_article(inner_width, |formatted| {
        let reader = ReaderView {
            formatted,
            active_pane: app.active_pane,
            theme: &app.theme,
            scroll_offset: app.reader_scroll_offset,
            is_zen_mode,
            show_icons: app.config.show_icons,
            padding: app.config.padding,
        };
        frame.render_widget(reader, area);
    });
}

/// The single chrome line: brand on the left, then sync/toast state, optional
/// compact shortcut hints, and layout/theme info on the right.
fn render_status_bar(app: &App, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let icons = app.config.show_icons;

    // One call rather than one per cell.
    buf.set_style(
        area,
        Style::default().bg(theme.status_bar_bg).fg(theme.status_bar_fg),
    );

    let brand = if icons { " 📰 RataRSS " } else { " RataRSS " };
    let mut spans = vec![Span::styled(
        brand,
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )];

    // Transient state (toast or sync) takes the slot right of the brand; the
    // pane-name text that used to sit here is gone.
    if let Some(ref msg) = app.toast_message {
        spans.push(Span::styled(
            format!(" {msg} "),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));
    } else if app.is_syncing {
        let spinner = get_spinner_frame(app.tick_count);
        spans.push(Span::styled(
            format!(" {spinner} syncing "),
            Style::default().fg(theme.warning_fg).add_modifier(Modifier::BOLD),
        ));
    } else if app.config.show_help_hints {
        spans.push(Span::styled(
            format!(" {HELP_HINTS} "),
            Style::default().fg(theme.fg_subtle),
        ));
    }

    let left_line = Line::from(spans);
    buf.set_line(area.x, area.y, &left_line, area.width);

    // Right: layout split and theme name, dropped entirely when the terminal
    // is too narrow to hold it without colliding with the brand.
    let right_text = format!(
        " {}/{}/{}  {} ",
        app.sidebar_width_percent,
        app.article_width_percent,
        app.reader_width_percent,
        theme.config.name
    );
    let right_len = unicode_width::UnicodeWidthStr::width(right_text.as_str()) as u16;
    let left_len = left_line.width() as u16;
    if area.width > right_len + left_len + 2 {
        let right_x = area.x + area.width - right_len;
        buf.set_string(
            right_x,
            area.y,
            &right_text,
            Style::default().fg(theme.fg_subtle),
        );
    }
}
