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

pub fn render_app(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let theme = &app.theme;

    // Outer layout: Top Header Bar (1 line), Main 3-Pane Body, Bottom Status Bar (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top Header Bar
            Constraint::Min(10),   // 3-Pane Body
            Constraint::Length(1), // Bottom Status Bar
        ])
        .split(area);

    let top_bar_area = chunks[0];
    let body_area = chunks[1];
    let status_bar_area = chunks[2];

    // --- Render Top Header Bar ---
    render_header_bar(app, top_bar_area, frame.buffer_mut());

    // --- Render Main Body ---
    if app.is_zen_mode {
        // Fullscreen active pane (usually reader or articles)
        match app.active_pane {
            ActivePane::Sidebar => {
                let sidebar = SidebarView {
                    items: &app.sidebar_items,
                    selected_index: app.sidebar_selected_idx,
                    active_pane: app.active_pane,
                    theme,
                    scroll_offset: app.sidebar_scroll_offset,
                };
                frame.render_widget(sidebar, body_area);
            }
            ActivePane::ArticleList => {
                let current_articles = app.get_filtered_articles();
                let article_list = ArticleListView {
                    articles: &current_articles,
                    selected_index: app.article_selected_idx,
                    active_pane: app.active_pane,
                    theme,
                    header_title: &app.current_view_title,
                    unread_count: app.current_view_unread_count,
                    search_query: &app.search_query,
                    is_searching: app.is_searching,
                    scroll_offset: app.article_scroll_offset,
                };
                frame.render_widget(article_list, body_area);
            }
            ActivePane::Reader => {
                let current_articles = app.get_filtered_articles();
                let selected_article = current_articles.get(app.article_selected_idx);
                let reader = ReaderView {
                    article: selected_article,
                    active_pane: app.active_pane,
                    theme,
                    scroll_offset: app.reader_scroll_offset,
                    is_zen_mode: true,
                };
                frame.render_widget(reader, body_area);
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

        // 1. Sidebar Pane
        let sidebar = SidebarView {
            items: &app.sidebar_items,
            selected_index: app.sidebar_selected_idx,
            active_pane: app.active_pane,
            theme,
            scroll_offset: app.sidebar_scroll_offset,
        };
        frame.render_widget(sidebar, pane_chunks[0]);

        // 2. Article List Pane
        let current_articles = app.get_filtered_articles();
        let article_list = ArticleListView {
            articles: &current_articles,
            selected_index: app.article_selected_idx,
            active_pane: app.active_pane,
            theme,
            header_title: &app.current_view_title,
            unread_count: app.current_view_unread_count,
            search_query: &app.search_query,
            is_searching: app.is_searching,
            scroll_offset: app.article_scroll_offset,
        };
        frame.render_widget(article_list, pane_chunks[1]);

        // 3. Reader View Pane
        let selected_article = current_articles.get(app.article_selected_idx);
        let reader = ReaderView {
            article: selected_article,
            active_pane: app.active_pane,
            theme,
            scroll_offset: app.reader_scroll_offset,
            is_zen_mode: false,
        };
        frame.render_widget(reader, pane_chunks[2]);
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
            file_path_input: &app.modal_export_path,
            status_msg: app.modal_export_status.as_deref(),
            theme,
        };
        frame.render_widget(modal, area);
    } else if app.show_theme_modal {
        let modal = ThemePickerModal {
            themes: &app.all_themes,
            selected_index: app.theme_picker_selected_idx,
            current_theme_name: &app.theme.config.name,
            theme,
        };
        frame.render_widget(modal, area);
    } else if app.show_help_modal {
        let modal = HelpModal { theme };
        frame.render_widget(modal, area);
    } else if app.show_delete_modal {
        let (name, is_folder) = app.get_delete_target_info();
        let modal = ConfirmDeleteModal {
            target_name: &name,
            is_folder,
            theme,
        };
        frame.render_widget(modal, area);
    }
}

fn render_header_bar(app: &App, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let bg_style = Style::default().bg(theme.status_bar_bg).fg(theme.fg);

    for x in area.x..area.x + area.width {
        buf.set_style(Rect::new(x, area.y, 1, 1), bg_style);
    }

    // App Brand / Title - RataRSS with no background badge so contrast is crisp and clean
    let app_brand = Span::styled(
        " 📰 RataRSS ",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    );

    // Sync status or Current View title
    let sync_span = if app.is_syncing {
        let spinner = get_spinner_frame(app.tick_count);
        Span::styled(
            format!("  {spinner} Fetching feeds... "),
            Style::default().fg(theme.warning_fg).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!("  Viewing: {} ", app.current_view_title),
            Style::default().fg(theme.fg_dim),
        )
    };

    let left_line = Line::from(vec![app_brand, sync_span]);
    buf.set_line(area.x, area.y, &left_line, area.width);

    // Right quick shortcuts
    let shortcuts = " [?] Help  [/] Config  [T] Themes  [a] Add  [r] Refresh  [f] Zen  [q] Quit ";
    let short_len = shortcuts.len() as u16;
    if area.width > short_len + 28 {
        let right_x = area.x + area.width - short_len;
        buf.set_string(
            right_x,
            area.y,
            shortcuts,
            Style::default().fg(theme.fg_subtle),
        );
    }
}

fn render_status_bar(app: &App, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;
    let bg_style = Style::default().bg(theme.status_bar_bg).fg(theme.status_bar_fg);

    for x in area.x..area.x + area.width {
        buf.set_style(Rect::new(x, area.y, 1, 1), bg_style);
    }

    // Left: Toast Message or Active Pane Info
    let left_text = if let Some(ref msg) = app.toast_message {
        format!(" 📢 {msg} ")
    } else {
        match app.active_pane {
            ActivePane::Sidebar => " [1: Feeds] Tab to Articles • j/k navigate • Enter select • a Add • d Delete • / Config".to_string(),
            ActivePane::ArticleList => " [2: Articles] Tab to Reader • j/k browse • m Read • s Star • Ctrl+F Search • / Config".to_string(),
            ActivePane::Reader => " [3: Reader] j/k/Space Scroll • o Browser • y Copy • f Zen • / Config".to_string(),
        }
    };

    buf.set_string(
        area.x,
        area.y,
        &left_text,
        if app.toast_message.is_some() {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.status_bar_fg)
        },
    );

    // Right: Pane Split info / Theme name
    let right_text = format!(
        " Split: {}%/{}%/{}%  Theme: {} ",
        app.sidebar_width_percent,
        app.article_width_percent,
        app.reader_width_percent,
        theme.config.name
    );
    let right_len = right_text.len() as u16;
    if area.width > right_len + 40 {
        let right_x = area.x + area.width - right_len;
        buf.set_string(
            right_x,
            area.y,
            &right_text,
            Style::default().fg(theme.fg_subtle),
        );
    }
}
