use crate::config::AppConfig;
use crate::fetcher::FeedFetcher;
use crate::model::{ActivePane, Article, CurrentFilter, Feed, SidebarItem, SmartFeedKind};
use crate::opml::{export_opml, parse_opml_file};
use crate::storage::{Database, UnreadCounts};
use crate::theme::Theme;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use tokio::runtime::Handle;

#[allow(dead_code)]
pub enum AppEvent {

    FeedRefreshFinished {
        feed_id: String,
        feed: Option<Feed>,
        articles: Vec<Article>,
        error: Option<String>,
    },
    AddFeedFinished {
        feed: Option<Feed>,
        articles: Vec<Article>,
        error: Option<String>,
    },
}

pub struct App {
    pub db: Database,
    pub config: AppConfig,
    pub theme: Theme,
    pub all_themes: Vec<Theme>,
    pub fetcher: FeedFetcher,
    pub tokio_handle: Handle,

    // Layout & Panes
    pub active_pane: ActivePane,
    pub sidebar_width_percent: u16,
    pub article_width_percent: u16,
    pub reader_width_percent: u16,
    pub is_zen_mode: bool,

    // Sidebar State
    pub feeds: Vec<Feed>,
    pub sidebar_items: Vec<SidebarItem>,
    pub sidebar_selected_idx: usize,
    pub sidebar_scroll_offset: usize,
    pub unread_counts: UnreadCounts,

    // Article List State
    pub current_filter: CurrentFilter,
    pub current_view_title: String,
    pub current_view_unread_count: usize,
    pub articles: Vec<Article>,
    pub article_selected_idx: usize,
    pub article_scroll_offset: usize,

    // Reader State
    pub reader_scroll_offset: usize,

    // Search & Filter State
    pub search_query: String,
    pub is_searching: bool,

    // Modals
    pub show_add_modal: bool,
    pub modal_url_input: String,
    pub modal_folder_input: String,
    pub modal_is_opml_mode: bool,
    pub modal_focused_field: usize,
    pub modal_error: Option<String>,
    pub modal_is_loading: bool,

    pub show_export_modal: bool,
    pub modal_export_path: String,
    pub modal_export_status: Option<String>,

    pub show_theme_modal: bool,
    pub theme_picker_selected_idx: usize,

    pub show_help_modal: bool,
    pub show_delete_modal: bool,

    // Async & Sync Status
    pub is_syncing: bool,
    pub tick_count: usize,
    pub toast_message: Option<String>,
    pub toast_time: Option<Instant>,
    pub event_sender: Sender<AppEvent>,
    pub event_receiver: Receiver<AppEvent>,

    pub should_quit: bool,
}

impl App {
    pub fn new(tokio_handle: Handle) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Database::new()?;
        let config = AppConfig::load();
        let theme = Theme::by_name(&config.theme);
        let all_themes = Theme::all_presets();
        let fetcher = FeedFetcher::new();
        let (event_sender, event_receiver) = channel();

        let mut app = Self {
            db,
            config: config.clone(),
            theme,
            all_themes,
            fetcher,
            tokio_handle,

            active_pane: ActivePane::Sidebar,
            sidebar_width_percent: config.sidebar_ratio,
            article_width_percent: config.article_list_ratio,
            reader_width_percent: config.reader_ratio,
            is_zen_mode: false,

            feeds: Vec::new(),
            sidebar_items: Vec::new(),
            sidebar_selected_idx: 1, // Start on 'Today'
            sidebar_scroll_offset: 0,
            unread_counts: UnreadCounts::default(),

            current_filter: CurrentFilter::Smart(SmartFeedKind::Today),
            current_view_title: "Today".to_string(),
            current_view_unread_count: 0,
            articles: Vec::new(),
            article_selected_idx: 0,
            article_scroll_offset: 0,

            reader_scroll_offset: 0,

            search_query: String::new(),
            is_searching: false,

            show_add_modal: false,
            modal_url_input: String::new(),
            modal_folder_input: String::new(),
            modal_is_opml_mode: false,
            modal_focused_field: 0,
            modal_error: None,
            modal_is_loading: false,

            show_export_modal: false,
            modal_export_path: "subscriptions.opml".to_string(),
            modal_export_status: None,

            show_theme_modal: false,
            theme_picker_selected_idx: 0,

            show_help_modal: false,
            show_delete_modal: false,

            is_syncing: false,
            tick_count: 0,
            toast_message: None,
            toast_time: None,
            event_sender,
            event_receiver,

            should_quit: false,
        };

        app.reload_data();

        if config.auto_refresh_on_startup {
            app.refresh_all_feeds();
        }

        Ok(app)
    }

    pub fn reload_data(&mut self) {
        if let Ok(feeds) = self.db.get_all_feeds() {
            self.feeds = feeds;
        }

        if let Ok(counts) = self.db.get_unread_counts() {
            self.unread_counts = counts;
        }

        self.rebuild_sidebar_items();
        self.reload_articles();
    }

    pub fn rebuild_sidebar_items(&mut self) {
        let mut items = Vec::new();

        // 1. Smart Feeds Header & Items
        items.push(SidebarItem::SmartHeader);
        items.push(SidebarItem::Smart(SmartFeedKind::Today, self.unread_counts.today));
        items.push(SidebarItem::Smart(SmartFeedKind::AllUnread, self.unread_counts.all_unread));
        items.push(SidebarItem::Smart(SmartFeedKind::Starred, self.unread_counts.starred));
        items.push(SidebarItem::Smart(SmartFeedKind::AllArticles, self.unread_counts.all_articles));

        // 2. Custom Folders and Feeds
        let mut folder_map: std::collections::BTreeMap<Option<String>, Vec<Feed>> = std::collections::BTreeMap::new();
        for f in &self.feeds {
            folder_map.entry(f.folder.clone()).or_default().push(f.clone());
        }

        for (folder_opt, mut folder_feeds) in folder_map {
            match folder_opt {
                Some(folder_name) => {
                    let is_expanded = self.db.is_folder_expanded(&folder_name);
                    let (folder_unread, _) = self.unread_counts.folder_counts.get(&folder_name).cloned().unwrap_or((0, 0));

                    items.push(SidebarItem::FolderHeader {
                        name: folder_name.clone(),
                        is_expanded,
                        unread_count: folder_unread,
                        feed_count: folder_feeds.len(),
                    });

                    if is_expanded {
                        folder_feeds.sort_by(|a, b| a.title.cmp(&b.title));
                        for f in folder_feeds {
                            let (unread, _) = self.unread_counts.feed_counts.get(&f.id).cloned().unwrap_or((0, 0));
                            items.push(SidebarItem::Feed {
                                feed_id: f.id.clone(),
                                title: f.title.clone(),
                                folder: Some(folder_name.clone()),
                                unread_count: unread,
                                has_error: f.error.is_some(),
                            });
                        }
                    }
                }
                None => {
                    folder_feeds.sort_by(|a, b| a.title.cmp(&b.title));
                    for f in folder_feeds {
                        let (unread, _) = self.unread_counts.feed_counts.get(&f.id).cloned().unwrap_or((0, 0));
                        items.push(SidebarItem::Feed {
                            feed_id: f.id.clone(),
                            title: f.title.clone(),
                            folder: None,
                            unread_count: unread,
                            has_error: f.error.is_some(),
                        });
                    }
                }
            }
        }

        self.sidebar_items = items;
        if self.sidebar_selected_idx >= self.sidebar_items.len() && !self.sidebar_items.is_empty() {
            self.sidebar_selected_idx = self.sidebar_items.len() - 1;
        }
    }

    pub fn reload_articles(&mut self) {
        if let Ok(articles) = self.db.get_articles_by_filter(&self.current_filter) {
            self.articles = articles;
        } else {
            self.articles.clear();
        }

        self.current_view_unread_count = self.articles.iter().filter(|a| !a.read).count();

        // Adjust selected index
        if self.article_selected_idx >= self.articles.len() && !self.articles.is_empty() {
            self.article_selected_idx = self.articles.len() - 1;
        }

        self.reader_scroll_offset = 0;

        // Auto mark read if configured and in reader pane
        if self.config.mark_read_on_open && self.active_pane == ActivePane::Reader {
            self.mark_current_article_read();
        }
    }

    pub fn get_filtered_articles(&self) -> Vec<Article> {
        if self.search_query.trim().is_empty() {
            return self.articles.clone();
        }

        let q = self.search_query.to_lowercase();
        self.articles
            .iter()
            .filter(|a| {
                a.title.to_lowercase().contains(&q)
                    || a.feed_title.to_lowercase().contains(&q)
                    || a.summary.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || a.author.as_deref().unwrap_or("").to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }

    pub fn on_tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);

        // Process background events from feed fetching
        let mut had_updates = false;
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                AppEvent::FeedRefreshFinished {
                    feed_id: _,
                    feed,
                    articles,
                    error,
                } => {
                    if let Some(f) = feed {
                        let _ = self.db.add_or_update_feed(&f);
                    }
                    if !articles.is_empty() {
                        let _ = self.db.insert_articles(&articles);
                        had_updates = true;
                    }
                    if let Some(err) = error {
                        self.set_toast(format!("Feed error: {err}"));
                    }
                }
                AppEvent::AddFeedFinished {
                    feed,
                    articles,
                    error,
                } => {
                    self.modal_is_loading = false;
                    if let Some(f) = feed {
                        let title = f.title.clone();
                        let _ = self.db.add_or_update_feed(&f);
                        if !articles.is_empty() {
                            let _ = self.db.insert_articles(&articles);
                        }
                        self.show_add_modal = false;
                        self.set_toast(format!("Added feed: {title} ({} articles)", articles.len()));
                        had_updates = true;
                    } else if let Some(err) = error {
                        self.modal_error = Some(err);
                    }
                }
            }
        }

        if had_updates {
            self.reload_data();
        }

        // Auto dismiss toast after 4 seconds
        if let Some(time) = self.toast_time {
            if time.elapsed().as_secs() > 4 {
                self.toast_message = None;
                self.toast_time = None;
            }
        }
    }

    pub fn set_toast<S: Into<String>>(&mut self, msg: S) {
        self.toast_message = Some(msg.into());
        self.toast_time = Some(Instant::now());
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Modal handlers
        if self.show_help_modal {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('?') || key.code == KeyCode::Char('q') {
                self.show_help_modal = false;
            }
            return;
        }

        if self.show_theme_modal {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => self.show_theme_modal = false,
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.theme_picker_selected_idx > 0 {
                        self.theme_picker_selected_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.theme_picker_selected_idx + 1 < self.all_themes.len() {
                        self.theme_picker_selected_idx += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(th) = self.all_themes.get(self.theme_picker_selected_idx) {
                        self.theme = th.clone();
                        self.config.theme = th.config.name.clone();
                        let _ = self.config.save();
                        self.set_toast(format!("Applied theme: {}", th.config.name));
                    }
                    self.show_theme_modal = false;
                }
                _ => {}
            }
            return;
        }

        if self.show_add_modal {
            match key.code {
                KeyCode::Esc => {
                    self.show_add_modal = false;
                    self.modal_error = None;
                }
                KeyCode::Tab => {
                    if !self.modal_is_opml_mode {
                        self.modal_focused_field = (self.modal_focused_field + 1) % 2;
                    }
                }
                KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.modal_is_opml_mode = !self.modal_is_opml_mode;
                    self.modal_error = None;
                }
                KeyCode::Backspace => {
                    if self.modal_focused_field == 0 {
                        self.modal_url_input.pop();
                    } else {
                        self.modal_folder_input.pop();
                    }
                }
                KeyCode::Char(c) => {
                    if self.modal_focused_field == 0 {
                        self.modal_url_input.push(c);
                    } else {
                        self.modal_folder_input.push(c);
                    }
                }
                KeyCode::Enter => {
                    self.submit_add_modal();
                }
                _ => {}
            }
            return;
        }

        if self.show_export_modal {
            match key.code {
                KeyCode::Esc => self.show_export_modal = false,
                KeyCode::Backspace => {
                    self.modal_export_path.pop();
                }
                KeyCode::Char(c) => {
                    self.modal_export_path.push(c);
                }
                KeyCode::Enter => {
                    self.submit_export_modal();
                }
                _ => {}
            }
            return;
        }

        if self.show_delete_modal {
            match key.code {
                KeyCode::Esc | KeyCode::Char('n') => self.show_delete_modal = false,
                KeyCode::Enter | KeyCode::Char('y') => {
                    self.confirm_delete_selected();
                    self.show_delete_modal = false;
                }
                _ => {}
            }
            return;
        }

        // Search mode input
        if self.is_searching {
            match key.code {
                KeyCode::Esc => {
                    self.is_searching = false;
                    self.search_query.clear();
                    self.article_selected_idx = 0;
                }
                KeyCode::Enter => {
                    self.is_searching = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.article_selected_idx = 0;
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.article_selected_idx = 0;
                }
                _ => {}
            }
            return;
        }

        // Global Keybindings
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.should_quit = true,
            KeyCode::Char('?') | KeyCode::F(1) => self.show_help_modal = true,
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.show_theme_modal = true;
                // find index of current theme
                self.theme_picker_selected_idx = self
                    .all_themes
                    .iter()
                    .position(|t| t.config.name == self.theme.config.name)
                    .unwrap_or(0);
            }
            KeyCode::Char('a') => {
                self.show_add_modal = true;
                self.modal_url_input.clear();
                self.modal_folder_input.clear();
                self.modal_is_opml_mode = false;
                self.modal_focused_field = 0;
                self.modal_error = None;
                self.modal_is_loading = false;
            }
            KeyCode::Char('e') => {
                self.show_export_modal = true;
                self.modal_export_status = None;
            }
            KeyCode::Char('d') => {
                if self.active_pane == ActivePane::Sidebar {
                    self.show_delete_modal = true;
                }
            }
            KeyCode::Char('f') | KeyCode::Char('z') => {
                self.is_zen_mode = !self.is_zen_mode;
            }

            // Pane switching
            KeyCode::Tab => self.active_pane = self.active_pane.next(),
            KeyCode::BackTab => self.active_pane = self.active_pane.prev(),
            KeyCode::Char('1') => self.active_pane = ActivePane::Sidebar,
            KeyCode::Char('2') => self.active_pane = ActivePane::ArticleList,
            KeyCode::Char('3') => self.active_pane = ActivePane::Reader,
            KeyCode::Char('h') | KeyCode::Left => self.active_pane = self.active_pane.prev(),
            KeyCode::Char('l') | KeyCode::Right => self.active_pane = self.active_pane.next(),

            // Pane resizing
            KeyCode::Char('<') => {
                if self.sidebar_width_percent > 12 {
                    self.sidebar_width_percent -= 2;
                    self.reader_width_percent += 2;
                }
            }
            KeyCode::Char('>') => {
                if self.sidebar_width_percent < 40 {
                    self.sidebar_width_percent += 2;
                    self.reader_width_percent = self.reader_width_percent.saturating_sub(2);
                }
            }
            KeyCode::Char('[') => {
                if self.article_width_percent > 18 {
                    self.article_width_percent -= 2;
                    self.reader_width_percent += 2;
                }
            }
            KeyCode::Char(']') => {
                if self.article_width_percent < 55 {
                    self.article_width_percent += 2;
                    self.reader_width_percent = self.reader_width_percent.saturating_sub(2);
                }
            }
            KeyCode::Char('+') => {
                if self.reader_width_percent < 75 {
                    self.reader_width_percent += 2;
                    self.article_width_percent = self.article_width_percent.saturating_sub(2);
                }
            }
            KeyCode::Char('-') => {
                if self.reader_width_percent > 20 {
                    self.reader_width_percent -= 2;
                    self.article_width_percent += 2;
                }
            }
            KeyCode::Char('=') => {
                self.sidebar_width_percent = self.config.sidebar_ratio;
                self.article_width_percent = self.config.article_list_ratio;
                self.reader_width_percent = self.config.reader_ratio;
                self.set_toast("Reset pane layout ratios");
            }

            // Search trigger
            KeyCode::Char('/') => {
                self.active_pane = ActivePane::ArticleList;
                self.is_searching = true;
                self.search_query.clear();
            }

            // Refresh feeds
            KeyCode::Char('r') => self.refresh_current_feed(),
            KeyCode::Char('R') => self.refresh_all_feeds(),

            // Article actions
            KeyCode::Char('m') => self.toggle_current_article_read(),
            KeyCode::Char('M') => self.mark_all_in_view_read(),
            KeyCode::Char('s') => self.toggle_current_article_star(),
            KeyCode::Char('o') => self.open_current_article_in_browser(),
            KeyCode::Char('y') => self.copy_current_article_url(),

            // Navigation within focused pane
            KeyCode::Char('j') | KeyCode::Down => self.navigate_down(),
            KeyCode::Char('k') | KeyCode::Up => self.navigate_up(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::PageUp => self.page_up(),
            KeyCode::Char(' ') => self.space_advance(),
            KeyCode::Char('g') | KeyCode::Home => self.jump_to_top(),
            KeyCode::Char('G') | KeyCode::End => self.jump_to_bottom(),
            KeyCode::Enter => self.handle_enter_key(),

            _ => {}
        }
    }

    pub fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollDown => self.navigate_down(),
            MouseEventKind::ScrollUp => self.navigate_up(),
            MouseEventKind::Down(MouseButton::Left) => {
                // Focus pane based on approximate x position
                // (Mouse clicks will set focus)
            }
            _ => {}
        }
    }

    fn navigate_down(&mut self) {
        match self.active_pane {
            ActivePane::Sidebar => {
                if self.sidebar_selected_idx + 1 < self.sidebar_items.len() {
                    self.sidebar_selected_idx += 1;
                    // Skip header items if any
                    if matches!(self.sidebar_items.get(self.sidebar_selected_idx), Some(SidebarItem::SmartHeader)) {
                        self.sidebar_selected_idx += 1;
                    }
                    self.apply_sidebar_selection();
                }
            }
            ActivePane::ArticleList => {
                let filtered = self.get_filtered_articles();
                if self.article_selected_idx + 1 < filtered.len() {
                    self.article_selected_idx += 1;
                    self.reader_scroll_offset = 0;
                    if self.config.mark_read_on_open {
                        self.mark_current_article_read();
                    }
                }
            }
            ActivePane::Reader => {
                self.reader_scroll_offset = self.reader_scroll_offset.saturating_add(2);
            }
        }
    }

    fn navigate_up(&mut self) {
        match self.active_pane {
            ActivePane::Sidebar => {
                if self.sidebar_selected_idx > 0 {
                    self.sidebar_selected_idx -= 1;
                    if matches!(self.sidebar_items.get(self.sidebar_selected_idx), Some(SidebarItem::SmartHeader)) {
                        if self.sidebar_selected_idx > 0 {
                            self.sidebar_selected_idx -= 1;
                        }
                    }
                    self.apply_sidebar_selection();
                }
            }
            ActivePane::ArticleList => {
                if self.article_selected_idx > 0 {
                    self.article_selected_idx -= 1;
                    self.reader_scroll_offset = 0;
                    if self.config.mark_read_on_open {
                        self.mark_current_article_read();
                    }
                }
            }
            ActivePane::Reader => {
                self.reader_scroll_offset = self.reader_scroll_offset.saturating_sub(2);
            }
        }
    }

    fn page_down(&mut self) {
        match self.active_pane {
            ActivePane::Sidebar => {
                self.sidebar_selected_idx = (self.sidebar_selected_idx + 8).min(self.sidebar_items.len().saturating_sub(1));
                self.apply_sidebar_selection();
            }
            ActivePane::ArticleList => {
                let filtered = self.get_filtered_articles();
                self.article_selected_idx = (self.article_selected_idx + 6).min(filtered.len().saturating_sub(1));
                self.reader_scroll_offset = 0;
            }
            ActivePane::Reader => {
                self.reader_scroll_offset = self.reader_scroll_offset.saturating_add(15);
            }
        }
    }

    fn page_up(&mut self) {
        match self.active_pane {
            ActivePane::Sidebar => {
                self.sidebar_selected_idx = self.sidebar_selected_idx.saturating_sub(8);
                self.apply_sidebar_selection();
            }
            ActivePane::ArticleList => {
                self.article_selected_idx = self.article_selected_idx.saturating_sub(6);
                self.reader_scroll_offset = 0;
            }
            ActivePane::Reader => {
                self.reader_scroll_offset = self.reader_scroll_offset.saturating_sub(15);
            }
        }
    }

    fn space_advance(&mut self) {
        // Space scrolls down the article; if at end, jumps to next article
        self.reader_scroll_offset = self.reader_scroll_offset.saturating_add(12);
    }

    fn jump_to_top(&mut self) {
        match self.active_pane {
            ActivePane::Sidebar => {
                self.sidebar_selected_idx = 1;
                self.apply_sidebar_selection();
            }
            ActivePane::ArticleList => {
                self.article_selected_idx = 0;
                self.reader_scroll_offset = 0;
            }
            ActivePane::Reader => {
                self.reader_scroll_offset = 0;
            }
        }
    }

    fn jump_to_bottom(&mut self) {
        match self.active_pane {
            ActivePane::Sidebar => {
                self.sidebar_selected_idx = self.sidebar_items.len().saturating_sub(1);
                self.apply_sidebar_selection();
            }
            ActivePane::ArticleList => {
                let filtered = self.get_filtered_articles();
                self.article_selected_idx = filtered.len().saturating_sub(1);
                self.reader_scroll_offset = 0;
            }
            ActivePane::Reader => {
                self.reader_scroll_offset = self.reader_scroll_offset.saturating_add(100);
            }
        }
    }

    fn handle_enter_key(&mut self) {
        match self.active_pane {
            ActivePane::Sidebar => {
                if let Some(item) = self.sidebar_items.get(self.sidebar_selected_idx) {
                    if let SidebarItem::FolderHeader { name, .. } = item {
                        let _ = self.db.toggle_folder_expanded(name);
                        self.reload_data();
                        return;
                    }
                }
                self.apply_sidebar_selection();
                self.active_pane = ActivePane::ArticleList;
            }
            ActivePane::ArticleList => {
                self.active_pane = ActivePane::Reader;
                self.mark_current_article_read();
            }
            ActivePane::Reader => {
                self.open_current_article_in_browser();
            }
        }
    }

    fn apply_sidebar_selection(&mut self) {
        if let Some(item) = self.sidebar_items.get(self.sidebar_selected_idx) {
            match item {
                SidebarItem::SmartHeader => {}
                SidebarItem::Smart(kind, _) => {
                    self.current_filter = CurrentFilter::Smart(*kind);
                    self.current_view_title = kind.title().to_string();
                    self.article_selected_idx = 0;
                    self.reload_articles();
                }
                SidebarItem::FolderHeader { name, .. } => {
                    self.current_filter = CurrentFilter::Folder(name.clone());
                    self.current_view_title = format!("Folder: {name}");
                    self.article_selected_idx = 0;
                    self.reload_articles();
                }
                SidebarItem::Feed { feed_id, title, .. } => {
                    self.current_filter = CurrentFilter::Feed(feed_id.clone());
                    self.current_view_title = title.clone();
                    self.article_selected_idx = 0;
                    self.reload_articles();
                }
            }
        }
    }

    pub fn toggle_current_article_read(&mut self) {
        let filtered = self.get_filtered_articles();
        if let Some(article) = filtered.get(self.article_selected_idx) {
            let new_read = !article.read;
            let _ = self.db.set_article_read(&article.id, new_read);
            self.reload_data();
            let status = if new_read { "read" } else { "unread" };
            self.set_toast(format!("Marked article as {status}"));
        }
    }

    pub fn mark_current_article_read(&mut self) {
        let filtered = self.get_filtered_articles();
        if let Some(article) = filtered.get(self.article_selected_idx) {
            if !article.read {
                let _ = self.db.set_article_read(&article.id, true);
                self.reload_data();
            }
        }
    }

    pub fn toggle_current_article_star(&mut self) {
        let filtered = self.get_filtered_articles();
        if let Some(article) = filtered.get(self.article_selected_idx) {
            let new_starred = !article.starred;
            let _ = self.db.set_article_starred(&article.id, new_starred);
            self.reload_data();
            let status = if new_starred { "starred ★" } else { "unstarred" };
            self.set_toast(format!("Article {status}"));
        }
    }

    pub fn mark_all_in_view_read(&mut self) {
        let _ = self.db.mark_all_read_in_filter(&self.current_filter);
        self.reload_data();
        self.set_toast(format!("Marked all articles in {} as read", self.current_view_title));
    }

    pub fn open_current_article_in_browser(&mut self) {
        let filtered = self.get_filtered_articles();
        if let Some(article) = filtered.get(self.article_selected_idx) {
            let url = &article.url;
            if !url.is_empty() {
                if let Err(e) = open::that(url) {
                    self.set_toast(format!("Failed to open browser: {e}"));
                } else {
                    self.set_toast(format!("Opened in browser: {url}"));
                }
            }
        }
    }

    pub fn copy_current_article_url(&mut self) {
        let filtered = self.get_filtered_articles();
        if let Some(article) = filtered.get(self.article_selected_idx) {
            let url = &article.url;
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(url.clone());
                self.set_toast(format!("Copied URL to clipboard: {url}"));
                return;
            }
            self.set_toast(format!("URL: {url}"));
        }
    }

    pub fn refresh_current_feed(&mut self) {
        let feed_to_refresh = match &self.current_filter {
            CurrentFilter::Feed(feed_id) => self.feeds.iter().find(|f| &f.id == feed_id).cloned(),
            _ => None,
        };

        if let Some(feed) = feed_to_refresh {
            let feed_title = feed.title.clone();
            self.is_syncing = true;
            let fetcher = self.fetcher.clone();
            let tx = self.event_sender.clone();

            self.tokio_handle.spawn(async move {
                let res = fetcher.fetch_feed(&feed).await;
                match res {
                    Ok((updated_feed, articles)) => {
                        let _ = tx.send(AppEvent::FeedRefreshFinished {
                            feed_id: feed.id,
                            feed: Some(updated_feed),
                            articles,
                            error: None,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::FeedRefreshFinished {
                            feed_id: feed.id,
                            feed: None,
                            articles: Vec::new(),
                            error: Some(e),
                        });
                    }
                }
            });
            self.set_toast(format!("Refreshing {}...", feed_title));
        } else {
            self.refresh_all_feeds();
        }
    }

    pub fn refresh_all_feeds(&mut self) {
        self.is_syncing = true;
        let feeds = self.feeds.clone();
        let fetcher = self.fetcher.clone();
        let tx = self.event_sender.clone();

        self.tokio_handle.spawn(async move {
            for feed in feeds {
                let res = fetcher.fetch_feed(&feed).await;
                match res {
                    Ok((updated_feed, articles)) => {
                        let _ = tx.send(AppEvent::FeedRefreshFinished {
                            feed_id: feed.id,
                            feed: Some(updated_feed),
                            articles,
                            error: None,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::FeedRefreshFinished {
                            feed_id: feed.id,
                            feed: None,
                            articles: Vec::new(),
                            error: Some(e),
                        });
                    }
                }
            }
        });

        self.set_toast("Refreshing all feeds in background...");
    }

    pub fn submit_add_modal(&mut self) {
        let input = self.modal_url_input.trim().to_string();
        if input.is_empty() {
            self.modal_error = Some("Input cannot be empty".to_string());
            return;
        }

        if self.modal_is_opml_mode {
            // Import OPML file
            match parse_opml_file(&input) {
                Ok(feeds) => {
                    let count = feeds.len();
                    for f in feeds {
                        let _ = self.db.add_or_update_feed(&f);
                    }
                    self.show_add_modal = false;
                    self.reload_data();
                    self.refresh_all_feeds();
                    self.set_toast(format!("Successfully imported {count} feeds from OPML"));
                }
                Err(e) => {
                    self.modal_error = Some(e);
                }
            }
        } else {
            // Single Feed URL
            let folder = if self.modal_folder_input.trim().is_empty() {
                None
            } else {
                Some(self.modal_folder_input.trim().to_string())
            };

            self.modal_is_loading = true;
            self.modal_error = None;

            let fetcher = self.fetcher.clone();
            let tx = self.event_sender.clone();

            self.tokio_handle.spawn(async move {
                let res = fetcher.discover_or_create_feed(&input, folder).await;
                match res {
                    Ok((feed, articles)) => {
                        let _ = tx.send(AppEvent::AddFeedFinished {
                            feed: Some(feed),
                            articles,
                            error: None,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::AddFeedFinished {
                            feed: None,
                            articles: Vec::new(),
                            error: Some(e),
                        });
                    }
                }
            });
        }
    }

    pub fn submit_export_modal(&mut self) {
        let path = self.modal_export_path.trim();
        if path.is_empty() {
            return;
        }

        match export_opml(&self.feeds, "RataRSS Subscriptions") {
            Ok(xml) => {
                if let Err(e) = std::fs::write(path, xml) {
                    self.modal_export_status = Some(format!("Error saving file: {e}"));
                } else {
                    self.show_export_modal = false;
                    self.set_toast(format!("Exported {} feeds to {path}", self.feeds.len()));
                }
            }
            Err(e) => {
                self.modal_export_status = Some(format!("Export error: {e}"));
            }
        }
    }

    pub fn get_delete_target_info(&self) -> (String, bool) {
        if let Some(item) = self.sidebar_items.get(self.sidebar_selected_idx) {
            match item {
                SidebarItem::FolderHeader { name, .. } => (name.clone(), true),
                SidebarItem::Feed { title, .. } => (title.clone(), false),
                _ => ("Item".to_string(), false),
            }
        } else {
            ("Item".to_string(), false)
        }
    }

    pub fn confirm_delete_selected(&mut self) {
        if let Some(item) = self.sidebar_items.get(self.sidebar_selected_idx) {
            match item {
                SidebarItem::FolderHeader { name, .. } => {
                    let folder = name.clone();
                    let _ = self.db.delete_folder(&folder);
                    self.reload_data();
                    self.set_toast(format!("Deleted folder: {folder}"));
                }
                SidebarItem::Feed { feed_id, title, .. } => {
                    let id = feed_id.clone();
                    let title = title.clone();
                    let _ = self.db.delete_feed(&id);
                    self.reload_data();
                    self.set_toast(format!("Deleted feed: {title}"));
                }
                _ => {}
            }
        }
    }
}
