use crate::config::{AppConfig, KeyBindingsConfig};
use crate::fetcher::FeedFetcher;
use crate::model::{ActivePane, Article, ArticleSlice, CurrentFilter, Feed, SidebarItem, SmartFeedKind};
use crate::opml::{export_opml, parse_opml_file};
use crate::storage::{Database, UnreadCounts};
use crate::theme::Theme;
use crate::ui::modals::{ConfigSubView, ModalHelper};
use ratatui::layout::Rect;

use crate::fuzzy;
use crate::reader::{render_article_to_text, FormattedArticle};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::cell::{Cell, RefCell};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use tokio::runtime::Handle;

/// The reader pane's formatted output, kept between frames.
///
/// Formatting an article means decompressing its body, stripping HTML and
/// wrapping every paragraph. That was happening on every single frame — so
/// holding `j` in the reader re-parsed the whole document per keypress. The
/// cache is keyed on everything the output depends on, so scrolling reuses it
/// and any real change (different article, resized pane, new theme) misses.
pub struct ReaderCache {
    pub article_id: String,
    pub width: u16,
    pub theme_name: String,
    pub formatted: FormattedArticle,
}

#[allow(dead_code)]
pub enum AppEvent {
    FeedRefreshFinished {
        feed_id: String,
        success: bool,
        new_articles_count: usize,
        error: Option<String>,
    },
    AddFeedFinished {
        feed: Option<Feed>,
        articles: Vec<Article>,
        error: Option<String>,
    },
}

/// Rows listed in the keybindings sub-view.
const KEYBIND_ROWS: usize = 26;

/// Number of rows in the main configuration menu.
const CONFIG_MENU_ITEMS: usize = 11;

/// Largest content padding, in cells, offered by the settings menu.
const MAX_PADDING: u16 = 6;

/// Largest gap between article cards, in *half* rows, offered by the settings
/// menu (6 halves = 3 rows).
const MAX_ARTICLE_SPACING: u16 = 6;

/// Saturating add of a signed delta to an unread counter.
fn adjust(counter: &mut usize, delta: isize) {
    if delta >= 0 {
        *counter = counter.saturating_add(delta as usize);
    } else {
        *counter = counter.saturating_sub(delta.unsigned_abs());
    }
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
    /// Folders the user has collapsed. Cached because the sidebar is rebuilt
    /// after every read/star change and this would otherwise be a query per
    /// rebuild.
    collapsed_folders: std::collections::HashSet<String>,

    // Article List State
    pub current_filter: CurrentFilter,
    pub current_view_title: String,
    pub current_view_unread_count: usize,
    pub articles: Vec<Article>,
    /// Indices into `articles` that survive the current search, or `None` when
    /// there is no search and the list is shown as-is. Recomputed only when the
    /// query or the article list changes — never per frame.
    filtered: Option<Vec<u32>>,
    pub article_selected_idx: usize,
    pub article_scroll_offset: usize,

    // Reader State
    pub reader_scroll_offset: usize,
    reader_cache: RefCell<Option<ReaderCache>>,
    /// Where the reader last drew article text, recorded during render so a
    /// click can be mapped onto the link that was drawn under it.
    reader_text_area: Cell<Rect>,

    // Search & Filter State
    pub search_query: String,
    pub is_searching: bool,
    /// Fuzzy filter over the sidebar's feeds (`ctrl+f`).
    pub feed_search_query: String,
    pub is_searching_feeds: bool,

    // Modals & Popups
    pub show_config_modal: bool,
    pub config_menu_selected_idx: usize,
    pub config_menu_subview: ConfigSubView,
    pub config_keybind_selected_idx: usize,

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
    pub pending_fetches: usize,
    pub sync_start_time: Option<Instant>,
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
            collapsed_folders: std::collections::HashSet::new(),

            current_filter: CurrentFilter::Smart(SmartFeedKind::Today),
            current_view_title: "Today".to_string(),
            current_view_unread_count: 0,
            articles: Vec::new(),
            filtered: None,
            article_selected_idx: 0,
            article_scroll_offset: 0,

            reader_scroll_offset: 0,
            reader_cache: RefCell::new(None),
            reader_text_area: Cell::new(Rect::new(0, 0, 0, 0)),

            search_query: String::new(),
            is_searching: false,
            feed_search_query: String::new(),
            is_searching_feeds: false,

            show_config_modal: false,
            config_menu_selected_idx: 0,
            config_menu_subview: ConfigSubView::Main,
            config_keybind_selected_idx: 0,

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
            pending_fetches: 0,
            sync_start_time: None,
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

        self.collapsed_folders = self.db.get_collapsed_folders();

        if let Ok(counts) = self.db.get_unread_counts() {
            self.unread_counts = counts;
        }

        self.rebuild_sidebar_items();
        self.reload_articles();
    }

    pub fn rebuild_sidebar_items(&mut self) {
        // While the sidebar is being searched it shows a flat, best-first list
        // of matching feeds: smart views and folder grouping would only get in
        // the way of picking one.
        let query = self.feed_search_query.trim().to_lowercase();
        if !query.is_empty() {
            let mut scored: Vec<(i32, &Feed)> = Vec::new();
            for f in &self.feeds {
                let folder = f.folder.as_deref().unwrap_or("");
                if let Some(score) = fuzzy::score_any([f.title.as_str(), folder], &query) {
                    scored.push((score, f));
                }
            }
            scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.title.cmp(&b.1.title)));

            self.sidebar_items = scored
                .into_iter()
                .map(|(_, f)| {
                    let (unread, _) = self
                        .unread_counts
                        .feed_counts
                        .get(&f.id)
                        .cloned()
                        .unwrap_or((0, 0));
                    SidebarItem::Feed {
                        feed_id: f.id.clone(),
                        title: f.title.clone(),
                        folder: f.folder.clone(),
                        unread_count: unread,
                        has_error: f.error.is_some(),
                    }
                })
                .collect();

            if self.sidebar_selected_idx >= self.sidebar_items.len() {
                self.sidebar_selected_idx = self.sidebar_items.len().saturating_sub(1);
            }
            return;
        }

        let mut items = Vec::new();

        // 1. Smart Feeds Header & Items
        items.push(SidebarItem::SmartHeader);
        items.push(SidebarItem::Smart(SmartFeedKind::Today, self.unread_counts.today));
        items.push(SidebarItem::Smart(SmartFeedKind::AllUnread, self.unread_counts.all_unread));
        items.push(SidebarItem::Smart(SmartFeedKind::Starred, self.unread_counts.starred));
        items.push(SidebarItem::Smart(SmartFeedKind::AllArticles, self.unread_counts.all_articles));

        // 2. Custom Folders and Feeds
        //
        // Group by borrowing the feeds rather than cloning each one into the
        // map — this runs on every data reload.
        let mut folder_map: std::collections::BTreeMap<Option<&str>, Vec<&Feed>> =
            std::collections::BTreeMap::new();
        for f in &self.feeds {
            folder_map.entry(f.folder.as_deref()).or_default().push(f);
        }

        for (folder_opt, mut folder_feeds) in folder_map {
            match folder_opt {
                Some(folder_name) => {
                    let is_expanded = !self.collapsed_folders.contains(folder_name);
                    let (folder_unread, _) = self.unread_counts.folder_counts.get(folder_name).cloned().unwrap_or((0, 0));

                    items.push(SidebarItem::FolderHeader {
                        name: folder_name.to_string(),
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
                                folder: Some(folder_name.to_string()),
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
            self.articles.shrink_to_fit();
        }

        self.recompute_filter();
        self.invalidate_reader_cache();
        self.current_view_unread_count = self.articles.iter().filter(|a| !a.read).count();

        // Adjust selected index
        let visible = self.visible_len();
        if self.article_selected_idx >= visible && visible > 0 {
            self.article_selected_idx = visible - 1;
        }

        self.reader_scroll_offset = 0;

        // Auto mark read if configured and in reader pane
        if self.config.mark_read_on_open && self.active_pane == ActivePane::Reader {
            self.mark_current_article_read();
        }
    }

    /// Recompute which articles the search matches.
    ///
    /// Called when the query or the underlying list changes — not on every
    /// access. With no query the result is `None`, meaning "all of `articles`",
    /// so the common case stores nothing at all.
    fn recompute_filter(&mut self) {
        let query = self.search_query.trim();
        if query.is_empty() {
            self.filtered = None;
            return;
        }

        let needle: String = query.to_lowercase();

        // Fuzzy-match each article, keeping the score so results can be ranked
        // by how well they matched rather than by position in the feed.
        let mut scored: Vec<(i32, u32)> = Vec::new();
        for (idx, a) in self.articles.iter().enumerate() {
            let score = fuzzy::score_any(
                [
                    a.title.as_str(),
                    a.feed_title.as_str(),
                    a.summary.as_deref().unwrap_or(""),
                    a.author.as_deref().unwrap_or(""),
                ],
                &needle,
            );
            if let Some(score) = score {
                scored.push((score, idx as u32));
            }
        }

        // Best score first; ties keep the list's own order, which is newest
        // first, so equally good matches stay chronological.
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));

        let mut matches: Vec<u32> = scored.into_iter().map(|(_, idx)| idx).collect();
        matches.shrink_to_fit();
        self.filtered = Some(matches);
    }

    /// Number of articles currently visible in the list pane.
    pub fn visible_len(&self) -> usize {
        match &self.filtered {
            Some(idx) => idx.len(),
            None => self.articles.len(),
        }
    }

    /// Borrow the visible article at `pos` in the list pane.
    pub fn visible_article(&self, pos: usize) -> Option<&Article> {
        match &self.filtered {
            Some(idx) => idx.get(pos).and_then(|i| self.articles.get(*i as usize)),
            None => self.articles.get(pos),
        }
    }

    /// Index into `self.articles` of the visible article at `pos`.
    fn source_index(&self, pos: usize) -> Option<usize> {
        match &self.filtered {
            Some(idx) => idx.get(pos).map(|i| *i as usize),
            None => (pos < self.articles.len()).then_some(pos),
        }
    }

    pub fn selected_article(&self) -> Option<&Article> {
        self.visible_article(self.article_selected_idx)
    }

    /// The article list as the widgets see it: the full list plus the optional
    /// index of search matches, so nothing is copied to render a frame.
    pub fn visible_articles(&self) -> ArticleSlice<'_> {
        ArticleSlice {
            all: &self.articles,
            filtered: self.filtered.as_deref(),
        }
    }

    /// Update the search query and refresh the match set.
    pub fn set_search_query(&mut self, edit: impl FnOnce(&mut String)) {
        edit(&mut self.search_query);
        self.article_selected_idx = 0;
        self.article_scroll_offset = 0;
        self.recompute_filter();
    }

    /// Update the feed search query and rebuild the sidebar to match.
    pub fn set_feed_search(&mut self, edit: impl FnOnce(&mut String)) {
        edit(&mut self.feed_search_query);
        self.sidebar_selected_idx = 0;
        self.sidebar_scroll_offset = 0;
        self.rebuild_sidebar_items();
    }

    /// The formatted reader output for the current selection, rendering it only
    /// when the cache misses. Takes `&self` so it can run inside the draw
    /// closure; see [`ReaderCache`].
    pub fn with_formatted_article<R>(
        &self,
        width: u16,
        f: impl FnOnce(Option<&FormattedArticle>) -> R,
    ) -> R {
        let article = match self.selected_article() {
            Some(a) => a,
            None => return f(None),
        };

        let mut cache = self.reader_cache.borrow_mut();
        let fresh = cache.as_ref().is_some_and(|c| {
            c.article_id == article.id && c.width == width && c.theme_name == self.theme.config.name
        });

        if !fresh {
            // The body lives in SQLite, not in the article list; fetching it
            // here means only the article actually on screen is ever decompressed.
            let body = self.db.get_article_body(&article.id);
            let formatted = render_article_to_text(article, body.as_deref(), &self.theme, width);
            *cache = Some(ReaderCache {
                article_id: article.id.clone(),
                width,
                theme_name: self.theme.config.name.clone(),
                formatted,
            });
        }

        f(cache.as_ref().map(|c| &c.formatted))
    }

    /// Record the reader's text region; called from the draw path.
    pub fn set_reader_text_area(&self, area: Rect) {
        self.reader_text_area.set(area);
    }

    /// The URL drawn at a screen position in the reader, if any.
    fn reader_link_at(&self, col: u16, row: u16) -> Option<String> {
        let area = self.reader_text_area.get();
        if area.width == 0
            || area.height == 0
            || col < area.x
            || col >= area.x + area.width
            || row < area.y
            || row >= area.y + area.height
        {
            return None;
        }

        let line = self.reader_scroll_offset + (row - area.y) as usize;
        let cache = self.reader_cache.borrow();
        cache
            .as_ref()?
            .formatted
            .link_at(line, col - area.x)
            .map(str::to_string)
    }

    fn invalidate_reader_cache(&mut self) {
        self.reader_cache.get_mut().take();
    }

    pub fn on_tick(&mut self) -> bool {
        self.tick_count = self.tick_count.wrapping_add(1);

        let mut had_updates = false;
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                AppEvent::FeedRefreshFinished {
                    feed_id: _,
                    success,
                    new_articles_count: _,
                    error,
                } => {
                    self.pending_fetches = self.pending_fetches.saturating_sub(1);
                    if self.pending_fetches == 0 {
                        self.is_syncing = false;
                        self.sync_start_time = None;
                    }

                    if success {
                        had_updates = true;
                    }
                    if let Some(err) = error {
                        self.set_toast(format!("Feed sync notice: {err}"));
                    }
                }
                AppEvent::AddFeedFinished {
                    feed,
                    articles,
                    error,
                } => {
                    self.modal_is_loading = false;
                    self.pending_fetches = self.pending_fetches.saturating_sub(1);
                    if self.pending_fetches == 0 {
                        self.is_syncing = false;
                        self.sync_start_time = None;
                    }

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

        // Failsafe timeout for background syncing (15 seconds max)
        if self.is_syncing {
            if let Some(start) = self.sync_start_time {
                if start.elapsed().as_secs() > 15 {
                    self.is_syncing = false;
                    self.pending_fetches = 0;
                    self.sync_start_time = None;
                    self.set_toast("Feed sync completed.");
                    had_updates = true;
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
                had_updates = true;
            }
        }

        had_updates || self.is_syncing || self.modal_is_loading
    }

    /// Show or hide the compact shortcut hints in the status bar (`??`).
    pub fn toggle_help_hints(&mut self) {
        self.config.show_help_hints = !self.config.show_help_hints;
        let _ = self.config.save();
        let status = if self.config.show_help_hints { "shown" } else { "hidden" };
        self.set_toast(format!("Shortcut hints {status}"));
    }

    pub fn set_toast<S: Into<String>>(&mut self, msg: S) {
        self.toast_message = Some(msg.into());
        self.toast_time = Some(Instant::now());
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // 1. Help Modal Handler
        //
        // Pressing the help key a second time (`??`) closes the modal *and*
        // toggles the compact hint line in the status bar, so the shortcuts can
        // be summoned or dismissed without opening the settings menu.
        if self.show_help_modal {
            match key.code {
                KeyCode::Char('?') | KeyCode::F(1) => {
                    self.show_help_modal = false;
                    self.toggle_help_hints();
                }
                KeyCode::Esc | KeyCode::Char('q') => self.show_help_modal = false,
                _ => {}
            }
            return;
        }

        // 2. Configuration Menu Popup Modal Handler
        if self.show_config_modal {
            self.handle_config_modal_key(key);
            return;
        }

        // 3. Theme Picker Modal Handler
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

        // 4. Add Feed Modal Handler
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

        // 5. Export Modal Handler
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

        // 6. Delete Confirmation Modal Handler
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

        // 7a. Feed search input mode (sidebar)
        if self.is_searching_feeds {
            match key.code {
                KeyCode::Esc => {
                    self.is_searching_feeds = false;
                    self.set_feed_search(|q| q.clear());
                }
                KeyCode::Enter => {
                    // Commit the highlighted feed and get out of the way.
                    self.apply_sidebar_selection();
                    self.is_searching_feeds = false;
                    self.set_feed_search(|q| q.clear());
                    self.active_pane = ActivePane::ArticleList;
                }
                KeyCode::Backspace => {
                    self.set_feed_search(|q| {
                        q.pop();
                    });
                }
                // j/k are literal text while typing, so navigate with arrows.
                KeyCode::Down => {
                    if self.sidebar_selected_idx + 1 < self.sidebar_items.len() {
                        self.sidebar_selected_idx += 1;
                    }
                }
                KeyCode::Up => {
                    self.sidebar_selected_idx = self.sidebar_selected_idx.saturating_sub(1);
                }
                KeyCode::Char(c) => {
                    self.set_feed_search(|q| q.push(c));
                }
                _ => {}
            }
            return;
        }

        // 7. Search input mode
        if self.is_searching {
            match key.code {
                KeyCode::Esc => {
                    self.is_searching = false;
                    self.set_search_query(|q| q.clear());
                }
                KeyCode::Enter => {
                    self.is_searching = false;
                }
                KeyCode::Backspace => {
                    self.set_search_query(|q| {
                        q.pop();
                    });
                }
                KeyCode::Char(c) => {
                    self.set_search_query(|q| q.push(c));
                }
                _ => {}
            }
            return;
        }

        // 8. Configurable Global & Pane Keybindings
        let kb = &self.config.keybindings;

        if kb.matches(&key, &kb.toggle_config) {
            self.show_config_modal = !self.show_config_modal;
            self.config_menu_subview = ConfigSubView::Main;
            return;
        }

        if kb.matches(&key, &kb.quit) {
            self.should_quit = true;
            return;
        }

        if kb.matches(&key, &kb.help) {
            self.show_help_modal = true;
            return;
        }

        if kb.matches(&key, &kb.theme_picker) {
            self.show_theme_modal = true;
            self.theme_picker_selected_idx = self
                .all_themes
                .iter()
                .position(|t| t.config.name.eq_ignore_ascii_case(&self.theme.config.name))
                .unwrap_or(0);
            return;
        }

        if kb.matches(&key, &kb.add_feed) {
            self.show_add_modal = true;
            self.modal_url_input.clear();
            self.modal_folder_input.clear();
            self.modal_is_opml_mode = false;
            self.modal_focused_field = 0;
            self.modal_error = None;
            self.modal_is_loading = false;
            return;
        }

        if kb.matches(&key, &kb.export_opml) {
            self.show_export_modal = true;
            self.modal_export_status = None;
            return;
        }

        if kb.matches(&key, &kb.delete_item) {
            if self.active_pane == ActivePane::Sidebar {
                self.show_delete_modal = true;
            }
            return;
        }

        if kb.matches(&key, &kb.toggle_zen) {
            self.is_zen_mode = !self.is_zen_mode;
            return;
        }

        if kb.matches(&key, &kb.search) {
            self.active_pane = ActivePane::ArticleList;
            self.is_searching = true;
            self.set_search_query(|q| q.clear());
            return;
        }

        if kb.matches(&key, &kb.search_feeds) {
            self.active_pane = ActivePane::Sidebar;
            self.is_searching_feeds = true;
            self.set_feed_search(|q| q.clear());
            return;
        }

        if kb.matches(&key, &kb.refresh_current) {
            self.refresh_current_feed();
            return;
        }

        if kb.matches(&key, &kb.refresh_all) {
            self.refresh_all_feeds();
            return;
        }

        if kb.matches(&key, &kb.toggle_read) {
            self.toggle_current_article_read();
            return;
        }

        if kb.matches(&key, &kb.mark_all_read) {
            self.mark_all_in_view_read();
            return;
        }

        if kb.matches(&key, &kb.toggle_star) {
            self.toggle_current_article_star();
            return;
        }

        if kb.matches(&key, &kb.open_browser) {
            self.open_current_article_in_browser();
            return;
        }

        if kb.matches(&key, &kb.copy_url) {
            self.copy_current_article_url();
            return;
        }

        // Pane navigation
        if kb.matches(&key, &kb.focus_next_pane) {
            self.active_pane = self.active_pane.next();
            return;
        }
        if kb.matches(&key, &kb.focus_prev_pane) {
            self.active_pane = self.active_pane.prev();
            return;
        }
        if kb.matches(&key, &kb.focus_sidebar) {
            self.active_pane = ActivePane::Sidebar;
            return;
        }
        if kb.matches(&key, &kb.focus_article_list) {
            self.active_pane = ActivePane::ArticleList;
            return;
        }
        if kb.matches(&key, &kb.focus_reader) {
            self.active_pane = ActivePane::Reader;
            return;
        }

        // Pane resizing
        if kb.matches(&key, &kb.resize_sidebar_dec) {
            if self.sidebar_width_percent > 12 {
                self.sidebar_width_percent -= 2;
                self.reader_width_percent += 2;
            }
            return;
        }
        if kb.matches(&key, &kb.resize_sidebar_inc) {
            if self.sidebar_width_percent < 40 {
                self.sidebar_width_percent += 2;
                self.reader_width_percent = self.reader_width_percent.saturating_sub(2);
            }
            return;
        }
        if kb.matches(&key, &kb.resize_article_dec) {
            if self.article_width_percent > 18 {
                self.article_width_percent -= 2;
                self.reader_width_percent += 2;
            }
            return;
        }
        if kb.matches(&key, &kb.resize_article_inc) {
            if self.article_width_percent < 55 {
                self.article_width_percent += 2;
                self.reader_width_percent = self.reader_width_percent.saturating_sub(2);
            }
            return;
        }
        if kb.matches(&key, &kb.resize_reader_inc) {
            if self.reader_width_percent < 75 {
                self.reader_width_percent += 2;
                self.article_width_percent = self.article_width_percent.saturating_sub(2);
            }
            return;
        }
        if kb.matches(&key, &kb.resize_reader_dec) {
            if self.reader_width_percent > 20 {
                self.reader_width_percent -= 2;
                self.article_width_percent += 2;
            }
            return;
        }
        if kb.matches(&key, &kb.reset_layout) {
            self.sidebar_width_percent = self.config.sidebar_ratio;
            self.article_width_percent = self.config.article_list_ratio;
            self.reader_width_percent = self.config.reader_ratio;
            self.set_toast("Reset pane layout ratios");
            return;
        }

        // Navigation inside pane
        if kb.matches(&key, &kb.nav_down) {
            self.navigate_down();
            return;
        }
        if kb.matches(&key, &kb.nav_up) {
            self.navigate_up();
            return;
        }
        if kb.matches(&key, &kb.page_down) {
            self.page_down();
            return;
        }
        if kb.matches(&key, &kb.page_up) {
            self.page_up();
            return;
        }
        if kb.matches(&key, &kb.space_advance) {
            self.space_advance();
            return;
        }
        if kb.matches(&key, &kb.jump_top) {
            self.jump_to_top();
            return;
        }
        if kb.matches(&key, &kb.jump_bottom) {
            self.jump_to_bottom();
            return;
        }
        if kb.matches(&key, &kb.select_enter) {
            self.handle_enter_key();
            return;
        }
    }

    fn handle_config_modal_key(&mut self, key: KeyEvent) {
        match self.config_menu_subview {
            ConfigSubView::Keybindings => match key.code {
                KeyCode::Esc | KeyCode::Backspace => {
                    self.config_menu_subview = ConfigSubView::Main;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.config_keybind_selected_idx > 0 {
                        self.config_keybind_selected_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.config_keybind_selected_idx + 1 < KEYBIND_ROWS {
                        self.config_keybind_selected_idx += 1;
                    }
                }
                KeyCode::Char('r') => {
                    self.config.keybindings = KeyBindingsConfig::default();
                    let _ = self.config.save();
                    self.set_toast("Reset keybindings to defaults");
                }
                KeyCode::Char('q') => {
                    self.show_config_modal = false;
                }
                _ => {}
            },
            ConfigSubView::Main => match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('/') => {
                    self.show_config_modal = false;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.config_menu_selected_idx > 0 {
                        self.config_menu_selected_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.config_menu_selected_idx + 1 < CONFIG_MENU_ITEMS {
                        self.config_menu_selected_idx += 1;
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.modify_config_item(false);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.modify_config_item(true);
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.activate_config_item();
                }
                _ => {}
            },
        }
    }

    fn modify_config_item(&mut self, forward: bool) {
        match self.config_menu_selected_idx {
            0 => {
                // Cycle themes
                self.cycle_theme(forward);
            }
            1 => {
                // Auto-refresh on startup toggle
                self.config.auto_refresh_on_startup = !self.config.auto_refresh_on_startup;
                let _ = self.config.save();
                let status = if self.config.auto_refresh_on_startup { "Enabled" } else { "Disabled" };
                self.set_toast(format!("Auto-refresh on startup: {status}"));
            }
            2 => {
                // Refresh interval
                if forward {
                    self.config.refresh_interval_minutes = (self.config.refresh_interval_minutes + 5).min(180);
                } else {
                    self.config.refresh_interval_minutes = self.config.refresh_interval_minutes.saturating_sub(5).max(1);
                }
                let _ = self.config.save();
                self.set_toast(format!("Refresh interval: {} min", self.config.refresh_interval_minutes));
            }
            3 => {
                // Mark read on open
                self.config.mark_read_on_open = !self.config.mark_read_on_open;
                let _ = self.config.save();
                let status = if self.config.mark_read_on_open { "Enabled" } else { "Disabled" };
                self.set_toast(format!("Mark read on open: {status}"));
            }
            4 => {
                // Wrap article text
                self.config.wrap_article_text = !self.config.wrap_article_text;
                let _ = self.config.save();
                let status = if self.config.wrap_article_text { "Enabled" } else { "Disabled" };
                self.set_toast(format!("Wrap article text: {status}"));
            }
            5 => {
                // Show icons
                self.config.show_icons = !self.config.show_icons;
                let _ = self.config.save();
                let status = if self.config.show_icons { "Enabled" } else { "Disabled" };
                self.set_toast(format!("Show icons & badges: {status}"));
            }
            6 => {
                // Content padding inside each pane
                self.config.padding = if forward {
                    (self.config.padding + 1).min(MAX_PADDING)
                } else {
                    self.config.padding.saturating_sub(1)
                };
                let _ = self.config.save();
                self.invalidate_reader_cache();
                self.set_toast(format!("Padding: {} cell(s)", self.config.padding));
            }
            7 => {
                // Vertical gap between article cards
                self.config.article_spacing = if forward {
                    (self.config.article_spacing + 1).min(MAX_ARTICLE_SPACING)
                } else {
                    self.config.article_spacing.saturating_sub(1)
                };
                let _ = self.config.save();
                self.set_toast(format!(
                    "Article spacing: {} row(s)",
                    crate::ui::article_list::spacing_label(self.config.article_spacing)
                ));
            }
            8 => {
                self.toggle_help_hints();
            }
            9 => {
                // Layout percentages
                if forward {
                    if self.sidebar_width_percent < 35 {
                        self.sidebar_width_percent += 2;
                        self.reader_width_percent = self.reader_width_percent.saturating_sub(2);
                    }
                } else if self.sidebar_width_percent > 15 {
                    self.sidebar_width_percent -= 2;
                    self.reader_width_percent += 2;
                }
                self.config.sidebar_ratio = self.sidebar_width_percent;
                self.config.article_list_ratio = self.article_width_percent;
                self.config.reader_ratio = self.reader_width_percent;
                let _ = self.config.save();
            }
            10 => {
                // Keybindings subview
                self.config_menu_subview = ConfigSubView::Keybindings;
                self.config_keybind_selected_idx = 0;
            }
            _ => {}
        }
    }

    fn activate_config_item(&mut self) {
        match self.config_menu_selected_idx {
            0 => {
                self.show_config_modal = false;
                self.show_theme_modal = true;
                self.theme_picker_selected_idx = self
                    .all_themes
                    .iter()
                    .position(|t| t.config.name.eq_ignore_ascii_case(&self.theme.config.name))
                    .unwrap_or(0);
            }
            1 => {
                self.config.auto_refresh_on_startup = !self.config.auto_refresh_on_startup;
                let _ = self.config.save();
                let status = if self.config.auto_refresh_on_startup { "Enabled" } else { "Disabled" };
                self.set_toast(format!("Auto-refresh on startup: {status}"));
            }
            2 => {
                self.config.refresh_interval_minutes = match self.config.refresh_interval_minutes {
                    1..=5 => 15,
                    6..=15 => 30,
                    16..=30 => 60,
                    _ => 5,
                };
                let _ = self.config.save();
                self.set_toast(format!("Refresh interval: {} min", self.config.refresh_interval_minutes));
            }
            3 => {
                self.config.mark_read_on_open = !self.config.mark_read_on_open;
                let _ = self.config.save();
                let status = if self.config.mark_read_on_open { "Enabled" } else { "Disabled" };
                self.set_toast(format!("Mark read on open: {status}"));
            }
            4 => {
                self.config.wrap_article_text = !self.config.wrap_article_text;
                let _ = self.config.save();
                let status = if self.config.wrap_article_text { "Enabled" } else { "Disabled" };
                self.set_toast(format!("Wrap article text: {status}"));
            }
            5 => {
                self.config.show_icons = !self.config.show_icons;
                let _ = self.config.save();
                let status = if self.config.show_icons { "Enabled" } else { "Disabled" };
                self.set_toast(format!("Show icons & badges: {status}"));
            }
            6 => {
                // Cycle padding, wrapping back to none.
                self.config.padding = (self.config.padding + 1) % (MAX_PADDING + 1);
                let _ = self.config.save();
                self.invalidate_reader_cache();
                self.set_toast(format!("Padding: {} cell(s)", self.config.padding));
            }
            7 => {
                // Cycle spacing, wrapping back to flush.
                self.config.article_spacing =
                    (self.config.article_spacing + 1) % (MAX_ARTICLE_SPACING + 1);
                let _ = self.config.save();
                self.set_toast(format!(
                    "Article spacing: {} row(s)",
                    crate::ui::article_list::spacing_label(self.config.article_spacing)
                ));
            }
            8 => {
                self.toggle_help_hints();
            }
            9 => {
                self.sidebar_width_percent = 22;
                self.article_width_percent = 33;
                self.reader_width_percent = 45;
                self.config.sidebar_ratio = 22;
                self.config.article_list_ratio = 33;
                self.config.reader_ratio = 45;
                let _ = self.config.save();
                self.set_toast("Reset pane layout ratios to 22/33/45");
            }
            10 => {
                self.config_menu_subview = ConfigSubView::Keybindings;
                self.config_keybind_selected_idx = 0;
            }
            _ => {}
        }
    }

    fn cycle_theme(&mut self, forward: bool) {
        if self.all_themes.is_empty() {
            return;
        }
        let current_pos = self
            .all_themes
            .iter()
            .position(|t| t.config.name.eq_ignore_ascii_case(&self.theme.config.name))
            .unwrap_or(0);

        let next_idx = if forward {
            (current_pos + 1) % self.all_themes.len()
        } else if current_pos == 0 {
            self.all_themes.len() - 1
        } else {
            current_pos - 1
        };

        if let Some(th) = self.all_themes.get(next_idx) {
            self.theme = th.clone();
            self.config.theme = th.config.name.clone();
            let _ = self.config.save();
            self.set_toast(format!("Applied theme: {}", th.config.name));
        }
    }

    pub fn handle_mouse_event(&mut self, mouse: MouseEvent, width: u16, height: u16) {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                self.handle_mouse_scroll(mouse.column, mouse.row, width, height, true);
            }
            MouseEventKind::ScrollUp => {
                self.handle_mouse_scroll(mouse.column, mouse.row, width, height, false);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_mouse_click(mouse.column, mouse.row, width, height);
            }
            _ => {}
        }
    }

    fn handle_mouse_scroll(&mut self, col: u16, _row: u16, width: u16, _height: u16, down: bool) {
        if self.show_theme_modal {
            if down {
                if self.theme_picker_selected_idx + 1 < self.all_themes.len() {
                    self.theme_picker_selected_idx += 1;
                }
            } else if self.theme_picker_selected_idx > 0 {
                self.theme_picker_selected_idx -= 1;
            }
            return;
        }

        if self.show_config_modal {
            if self.config_menu_subview == ConfigSubView::Keybindings {
                if down {
                    if self.config_keybind_selected_idx + 1 < KEYBIND_ROWS {
                        self.config_keybind_selected_idx += 1;
                    }
                } else if self.config_keybind_selected_idx > 0 {
                    self.config_keybind_selected_idx -= 1;
                }
            } else {
                if down {
                    if self.config_menu_selected_idx + 1 < CONFIG_MENU_ITEMS {
                        self.config_menu_selected_idx += 1;
                    }
                } else if self.config_menu_selected_idx > 0 {
                    self.config_menu_selected_idx -= 1;
                }
            }
            return;
        }

        let sidebar_w = width * self.sidebar_width_percent / 100;
        let article_w = width * self.article_width_percent / 100;

        if self.is_zen_mode {
            match self.active_pane {
                ActivePane::Sidebar => {
                    if down { self.navigate_down(); } else { self.navigate_up(); }
                }
                ActivePane::ArticleList => {
                    if down { self.navigate_down(); } else { self.navigate_up(); }
                }
                ActivePane::Reader => {
                    if down {
                        self.reader_scroll_offset = self.reader_scroll_offset.saturating_add(3);
                    } else {
                        self.reader_scroll_offset = self.reader_scroll_offset.saturating_sub(3);
                    }
                }
            }
            return;
        }

        if col < sidebar_w {
            self.active_pane = ActivePane::Sidebar;
            if down { self.navigate_down(); } else { self.navigate_up(); }
        } else if col < sidebar_w + article_w {
            self.active_pane = ActivePane::ArticleList;
            if down { self.navigate_down(); } else { self.navigate_up(); }
        } else {
            self.active_pane = ActivePane::Reader;
            if down {
                self.reader_scroll_offset = self.reader_scroll_offset.saturating_add(3);
            } else {
                self.reader_scroll_offset = self.reader_scroll_offset.saturating_sub(3);
            }
        }
    }

    fn handle_mouse_click(&mut self, col: u16, row: u16, width: u16, height: u16) {
        // 1. Modals
        if self.show_theme_modal {
            let popup = ModalHelper::centered_rect(72, 65, Rect::new(0, 0, width, height));
            if col >= popup.x && col < popup.x + popup.width && row >= popup.y && row < popup.y + popup.height {
                let inner_y = popup.y + 1;
                let visible_height = popup.height.saturating_sub(2) as usize;
                if row >= inner_y && row < inner_y + (visible_height as u16) {
                    let mut scroll_offset = 0;
                    if self.theme_picker_selected_idx >= visible_height {
                        scroll_offset = self.theme_picker_selected_idx + 1 - visible_height;
                    }
                    if scroll_offset + visible_height > self.all_themes.len() {
                        scroll_offset = self.all_themes.len().saturating_sub(visible_height);
                    }
                    let clicked_row = (row - inner_y) as usize;
                    let target_idx = scroll_offset + clicked_row;
                    if let Some(th) = self.all_themes.get(target_idx) {
                        self.theme_picker_selected_idx = target_idx;
                        self.theme = th.clone();
                        self.config.theme = th.config.name.clone();
                        let _ = self.config.save();
                        self.set_toast(format!("Applied theme: {}", th.config.name));
                        self.show_theme_modal = false;
                    }
                }
            } else {
                self.show_theme_modal = false;
            }
            return;
        }

        if self.show_config_modal {
            if self.config_menu_subview == ConfigSubView::Keybindings {
                let popup = ModalHelper::centered_rect(80, 70, Rect::new(0, 0, width, height));
                if col >= popup.x && col < popup.x + popup.width && row >= popup.y && row < popup.y + popup.height {
                    let inner_y = popup.y + 2;
                    let visible_height = popup.height.saturating_sub(3) as usize;
                    if row >= inner_y && row < inner_y + (visible_height as u16) {
                        let clicked_row = (row - inner_y) as usize;
                        if clicked_row < KEYBIND_ROWS {
                            self.config_keybind_selected_idx = clicked_row;
                        }
                    }
                    let bottom_y = popup.y + popup.height.saturating_sub(2);
                    if row >= bottom_y {
                        self.config_menu_subview = ConfigSubView::Main;
                    }
                } else {
                    self.show_config_modal = false;
                }
            } else {
                let popup = ModalHelper::bottom_sheet_rect(88, 15, Rect::new(0, 0, width, height));
                if col >= popup.x && col < popup.x + popup.width && row >= popup.y && row < popup.y + popup.height {
                    let inner_y = popup.y + 1;
                    if row >= inner_y && row < inner_y + CONFIG_MENU_ITEMS as u16 {
                        let clicked_item = (row - inner_y) as usize;
                        self.config_menu_selected_idx = clicked_item;
                        if col > popup.x + 30 {
                            self.activate_config_item();
                        }
                    }
                } else {
                    self.show_config_modal = false;
                }
            }
            return;
        }

        if self.show_help_modal {
            self.show_help_modal = false;
            return;
        }

        if self.show_delete_modal {
            let popup = ModalHelper::centered_rect(50, 25, Rect::new(0, 0, width, height));
            if col >= popup.x && col < popup.x + popup.width && row >= popup.y && row < popup.y + popup.height {
                let bottom_y = popup.y + popup.height.saturating_sub(2);
                if row >= bottom_y {
                    if col < popup.x + 24 {
                        self.confirm_delete_selected();
                    }
                    self.show_delete_modal = false;
                }
            } else {
                self.show_delete_modal = false;
            }
            return;
        }

        if self.show_add_modal {
            let popup = ModalHelper::centered_rect(60, 45, Rect::new(0, 0, width, height));
            if col >= popup.x && col < popup.x + popup.width && row >= popup.y && row < popup.y + popup.height {
                if row >= popup.y + 3 && row <= popup.y + 5 {
                    self.modal_focused_field = 0;
                } else if row >= popup.y + 6 && row <= popup.y + 8 {
                    self.modal_focused_field = 1;
                } else if row >= popup.y + popup.height.saturating_sub(2) {
                    self.submit_add_modal();
                }
            } else {
                self.show_add_modal = false;
            }
            return;
        }

        if self.show_export_modal {
            let popup = ModalHelper::centered_rect(55, 30, Rect::new(0, 0, width, height));
            if col >= popup.x && col < popup.x + popup.width && row >= popup.y && row < popup.y + popup.height {
                if row >= popup.y + popup.height.saturating_sub(2) {
                    self.submit_export_modal();
                }
            } else {
                self.show_export_modal = false;
            }
            return;
        }

        // 2. Bottom Status Bar (`row == height - 1`) — the only chrome line left.
        if row == height.saturating_sub(1) {
            self.show_config_modal = !self.show_config_modal;
            self.config_menu_subview = ConfigSubView::Main;
            return;
        }

        // 3. Main Body Panes (`row < height - 1`; the body now starts at row 0)
        let sidebar_w = width * self.sidebar_width_percent / 100;
        let article_w = width * self.article_width_percent / 100;

        if self.is_zen_mode {
            match self.active_pane {
                ActivePane::Sidebar => self.handle_sidebar_click(row),
                ActivePane::ArticleList => self.handle_article_list_click(row),
                ActivePane::Reader => self.handle_reader_click(col, row, 0),
            }
            return;
        }

        if col < sidebar_w {
            self.active_pane = ActivePane::Sidebar;
            self.handle_sidebar_click(row);
        } else if col < sidebar_w + article_w {
            self.active_pane = ActivePane::ArticleList;
            self.handle_article_list_click(row);
        } else {
            self.active_pane = ActivePane::Reader;
            self.handle_reader_click(col, row, sidebar_w + article_w);
        }
    }

    fn handle_sidebar_click(&mut self, row: u16) {
        // Row 0 is the pane border, then the search bar when it is showing.
        let first_item_y = 1 + crate::ui::sidebar::SidebarView::search_rows(
            self.is_searching_feeds,
            &self.feed_search_query,
        );
        if row >= first_item_y {
            let clicked_row = (row - first_item_y) as usize + self.sidebar_scroll_offset;
            if clicked_row < self.sidebar_items.len() {
                self.sidebar_selected_idx = clicked_row;
                if let Some(SidebarItem::FolderHeader { name, .. }) = self.sidebar_items.get(clicked_row) {
                    let folder_name = name.clone();
                    let _ = self.db.toggle_folder_expanded(&folder_name);
                    self.reload_data();
                } else {
                    self.apply_sidebar_selection();
                }
            }
        }
    }

    fn handle_article_list_click(&mut self, row: u16) {
        let search_active = self.is_searching || !self.search_query.is_empty();
        // Border on row 0, then the search bar and its separator when active.
        let card_start_y = if search_active { 3 } else { 1 };

        if row >= 1 && row < card_start_y {
            self.is_searching = true;
            return;
        }

        if row >= card_start_y {
            let card_offset = crate::ui::article_list::card_at_row(
                row - card_start_y,
                self.config.article_spacing,
            );
            let target_idx = self.article_scroll_offset + card_offset;

            if target_idx < self.visible_len() {
                if target_idx == self.article_selected_idx {
                    self.active_pane = ActivePane::Reader;
                    self.mark_current_article_read();
                } else {
                    self.article_selected_idx = target_idx;
                    self.reader_scroll_offset = 0;
                    if self.config.mark_read_on_open {
                        self.mark_current_article_read();
                    }
                }
            }
        }
    }

    /// A click on a link opens it; anywhere else in the pane scrolls.
    fn handle_reader_click(&mut self, col: u16, row: u16, _pane_x: u16) {
        if let Some(url) = self.reader_link_at(col, row) {
            // Feeds carry relative and `javascript:` hrefs; handing those to
            // the OS opener does nothing useful, so say so instead.
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                self.set_toast(format!("Not an absolute link: {url}"));
                return;
            }
            if let Err(e) = open::that(&url) {
                self.set_toast(format!("Failed to open link: {e}"));
            } else {
                self.set_toast(format!("Opened link: {url}"));
            }
            return;
        }

        if row < 12 {
            self.reader_scroll_offset = self.reader_scroll_offset.saturating_sub(4);
        } else {
            self.reader_scroll_offset = self.reader_scroll_offset.saturating_add(4);
        }
    }

    fn navigate_down(&mut self) {
        match self.active_pane {
            ActivePane::Sidebar => {
                if self.sidebar_selected_idx + 1 < self.sidebar_items.len() {
                    self.sidebar_selected_idx += 1;
                    if matches!(self.sidebar_items.get(self.sidebar_selected_idx), Some(SidebarItem::SmartHeader)) {
                        self.sidebar_selected_idx += 1;
                    }
                    self.apply_sidebar_selection();
                }
            }
            ActivePane::ArticleList => {
                if self.article_selected_idx + 1 < self.visible_len() {
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
                self.article_selected_idx =
                    (self.article_selected_idx + 6).min(self.visible_len().saturating_sub(1));
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
                self.article_selected_idx = self.visible_len().saturating_sub(1);
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

    /// Apply a read/unread change to the database and patch the in-memory
    /// state to match.
    ///
    /// This used to call `reload_data()`, which re-queried every feed, every
    /// unread count and the entire article list. With `mark_read_on_open`
    /// enabled that ran on every `j`/`k`, so simply scrolling the list
    /// re-read the whole database — the single worst stall in the UI. Nothing
    /// about a read flag can change which articles are in the list except the
    /// "All Unread" view, which is handled explicitly below.
    fn apply_read_change(&mut self, pos: usize, read: bool) {
        let Some(src) = self.source_index(pos) else {
            return;
        };
        let Some(article) = self.articles.get_mut(src) else {
            return;
        };
        if article.read == read {
            return;
        }

        article.read = read;
        let id = article.id.clone();
        let feed_id = article.feed_id.clone();
        let _ = self.db.set_article_read(&id, read);

        let delta: isize = if read { -1 } else { 1 };
        adjust(&mut self.current_view_unread_count, delta);
        adjust(&mut self.unread_counts.all_unread, delta);
        adjust(&mut self.unread_counts.today, delta);
        if let Some((unread, _)) = self.unread_counts.feed_counts.get_mut(&feed_id) {
            adjust(unread, delta);
        }
        let folder = self
            .feeds
            .iter()
            .find(|f| f.id == feed_id)
            .and_then(|f| f.folder.clone());
        if let Some(folder) = folder {
            if let Some((unread, _)) = self.unread_counts.folder_counts.get_mut(&folder) {
                adjust(unread, delta);
            }
        }

        self.rebuild_sidebar_items();

        // "All Unread" is the one view whose membership depends on the flag, so
        // it does need the list rebuilt.
        if matches!(self.current_filter, CurrentFilter::Smart(SmartFeedKind::AllUnread)) && read {
            self.reload_articles();
        }
    }

    pub fn toggle_current_article_read(&mut self) {
        let Some(article) = self.selected_article() else {
            return;
        };
        let new_read = !article.read;
        self.apply_read_change(self.article_selected_idx, new_read);
        let status = if new_read { "read" } else { "unread" };
        self.set_toast(format!("Marked article as {status}"));
    }

    pub fn mark_current_article_read(&mut self) {
        self.apply_read_change(self.article_selected_idx, true);
    }

    pub fn toggle_current_article_star(&mut self) {
        let Some(src) = self.source_index(self.article_selected_idx) else {
            return;
        };
        let Some(article) = self.articles.get_mut(src) else {
            return;
        };

        let new_starred = !article.starred;
        article.starred = new_starred;
        let id = article.id.clone();
        let _ = self.db.set_article_starred(&id, new_starred);

        adjust(
            &mut self.unread_counts.starred,
            if new_starred { 1 } else { -1 },
        );
        self.rebuild_sidebar_items();

        // Same as read state: only the Starred view can lose or gain a row.
        if matches!(self.current_filter, CurrentFilter::Smart(SmartFeedKind::Starred)) {
            self.reload_articles();
        }

        let status = if new_starred { "starred ★" } else { "unstarred" };
        self.set_toast(format!("Article {status}"));
    }

    pub fn mark_all_in_view_read(&mut self) {
        let _ = self.db.mark_all_read_in_filter(&self.current_filter);
        self.reload_data();
        self.set_toast(format!("Marked all articles in {} as read", self.current_view_title));
    }

    pub fn open_current_article_in_browser(&mut self) {
        let Some(url) = self.selected_article().map(|a| a.url.clone()) else {
            return;
        };
        if url.is_empty() {
            return;
        }
        if let Err(e) = open::that(&url) {
            self.set_toast(format!("Failed to open browser: {e}"));
        } else {
            self.set_toast(format!("Opened in browser: {url}"));
        }
    }

    pub fn copy_current_article_url(&mut self) {
        let Some(url) = self.selected_article().map(|a| a.url.clone()) else {
            return;
        };
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(url.clone());
            self.set_toast(format!("Copied URL to clipboard: {url}"));
            return;
        }
        self.set_toast(format!("URL: {url}"));
    }

    pub fn refresh_current_feed(&mut self) {
        let feed_to_refresh = match &self.current_filter {
            CurrentFilter::Feed(feed_id) => self.feeds.iter().find(|f| &f.id == feed_id).cloned(),
            _ => None,
        };

        if let Some(feed) = feed_to_refresh {
            let feed_title = feed.title.clone();
            self.pending_fetches += 1;
            self.is_syncing = true;
            self.sync_start_time = Some(Instant::now());

            let fetcher = self.fetcher.clone();
            let tx = self.event_sender.clone();
            let db = self.db.clone();

            self.tokio_handle.spawn(async move {
                let res = fetcher.fetch_feed(&feed).await;
                match res {
                    Ok((updated_feed, articles)) => {
                        let count = articles.len();
                        let _ = db.add_or_update_feed(&updated_feed);
                        if !articles.is_empty() {
                            let _ = db.insert_articles(&articles);
                        }
                        let _ = tx.send(AppEvent::FeedRefreshFinished {
                            feed_id: feed.id,
                            success: true,
                            new_articles_count: count,
                            error: None,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::FeedRefreshFinished {
                            feed_id: feed.id,
                            success: false,
                            new_articles_count: 0,
                            error: Some(e),
                        });
                    }
                }
            });
            self.set_toast(format!("Refreshing {feed_title}..."));
        } else {
            self.refresh_all_feeds();
        }
    }

    pub fn refresh_all_feeds(&mut self) {
        if self.feeds.is_empty() {
            self.is_syncing = false;
            return;
        }

        self.pending_fetches += self.feeds.len();
        self.is_syncing = true;
        self.sync_start_time = Some(Instant::now());

        let feeds = self.feeds.clone();
        let fetcher = self.fetcher.clone();
        let tx = self.event_sender.clone();
        let db = self.db.clone();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(10)); // max 10 concurrent fetches

        for feed in feeds {
            let fetcher = fetcher.clone();
            let tx = tx.clone();
            let db = db.clone();
            let sem = semaphore.clone();

            self.tokio_handle.spawn(async move {
                let _permit = sem.acquire().await;
                let res = fetcher.fetch_feed(&feed).await;
                match res {
                    Ok((updated_feed, articles)) => {
                        let count = articles.len();
                        let _ = db.add_or_update_feed(&updated_feed);
                        if !articles.is_empty() {
                            let _ = db.insert_articles(&articles);
                        }
                        let _ = tx.send(AppEvent::FeedRefreshFinished {
                            feed_id: feed.id,
                            success: true,
                            new_articles_count: count,
                            error: None,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::FeedRefreshFinished {
                            feed_id: feed.id,
                            success: false,
                            new_articles_count: 0,
                            error: Some(e),
                        });
                    }
                }
            });
        }

        self.set_toast("Refreshing feeds in background...");
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
            self.pending_fetches += 1;
            self.is_syncing = true;
            self.sync_start_time = Some(Instant::now());

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
