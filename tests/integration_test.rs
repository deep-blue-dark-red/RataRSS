#[cfg(test)]
mod tests {
    use chrono::Utc;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use ratarss::config::{key_matches, AppConfig};
    use ratarss::model::Article;
    use ratarss::model::Feed;
    use ratarss::opml::{export_opml, parse_opml_str};
    use ratarss::reader::render_article_to_text;
    use ratarss::theme::Theme;

    #[test]
    fn test_opml_roundtrip() {
        let feeds = vec![
            Feed::new(
                "bloomberg".to_string(),
                "Bloomberg Markets".to_string(),
                "https://feeds.bloomberg.com/markets/news.rss".to_string(),
                Some("https://bloomberg.com".to_string()),
                Some("Fin Econ".to_string()),
            ),
            Feed::new(
                "hn".to_string(),
                "Hacker News".to_string(),
                "https://news.ycombinator.com/rss".to_string(),
                None,
                None,
            ),
        ];

        let xml = export_opml(&feeds, "Subscriptions").expect("export failed");
        assert!(xml.contains("Bloomberg Markets"));
        assert!(xml.contains("Fin Econ"));
        assert!(xml.contains("https://news.ycombinator.com/rss"));

        let parsed = parse_opml_str(&xml).expect("parse failed");
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_themes_load_correctly() {
        let presets = Theme::all_presets();
        assert!(presets.len() >= 25, "Should have 25+ rich presets");
        let dark = Theme::by_name("RataRSS Dark");
        assert_eq!(dark.config.name, "RataRSS Dark");
        let mocha = Theme::by_name("Catppuccin Mocha");
        assert_eq!(mocha.config.name, "Catppuccin Mocha");
        let tokyo = Theme::by_name("Tokyo Night");
        assert_eq!(tokyo.config.name, "Tokyo Night");
        let rose = Theme::by_name("Rosé Pine");
        assert_eq!(rose.config.name, "Rosé Pine");
        let cyber = Theme::by_name("Cyberpunk Neon");
        assert_eq!(cyber.config.name, "Cyberpunk Neon");
        let solar = Theme::by_name("Solarized Dark");
        assert_eq!(solar.config.name, "Solarized Dark");

        // Verify backward compatibility aliases
        let legacy_dark = Theme::by_name("NetNewsWire Dark");
        assert_eq!(legacy_dark.config.name, "RataRSS Dark");
    }

    #[test]
    fn test_reader_rendering() {
        let theme = Theme::ratarss_dark();
        let article = Article {
            id: "test-1".to_string(),
            feed_id: "test-feed".to_string(),
            feed_title: "Tech News".to_string(),
            title: "Rust 2026 Edition Released".to_string(),
            author: Some("Ferris".to_string()),
            summary: Some("Summary preview".to_string()),
            content: Some(
                "<p>Hello <strong>World</strong>!</p><blockquote>Quoted wisdom</blockquote>"
                    .to_string(),
            ),
            url: "https://example.com/rust".to_string(),
            published: Some(Utc::now()),
            read: false,
            starred: true,
            created_at: Utc::now(),
        };

        let formatted = render_article_to_text(&article, &theme, 80);
        assert!(formatted.total_lines > 0);
        assert!(formatted.links.contains(&"https://example.com/rust".to_string()));
    }

    #[test]
    fn test_database_operations_and_filters() {
        use ratarss::model::{CurrentFilter, SmartFeedKind};
        use ratarss::storage::Database;

        let db = Database::in_memory().expect("in memory db failed");
        let feeds = db.get_all_feeds().expect("get feeds failed");
        assert!(!feeds.is_empty(), "Database should seed with default feeds");

        let counts = db.get_unread_counts().expect("counts failed");
        assert!(counts.all_articles > 0);

        let today_articles = db
            .get_articles_by_filter(&CurrentFilter::Smart(SmartFeedKind::Today))
            .expect("today failed");
        assert!(!today_articles.is_empty());

        let first_id = &today_articles[0].id;
        db.set_article_read(first_id, true)
            .expect("mark read failed");
        let updated_counts = db.get_unread_counts().expect("updated counts failed");
        assert_eq!(
            updated_counts.all_unread,
            counts.all_unread.saturating_sub(1)
        );

        db.set_article_starred(first_id, true).expect("star failed");
        let starred = db
            .get_articles_by_filter(&CurrentFilter::Smart(SmartFeedKind::Starred))
            .expect("starred query failed");
        assert!(starred.iter().any(|a| a.id == *first_id));
    }

    #[test]
    fn test_snippet_and_html_cleaning() {
        use ratarss::fetcher::{clean_html_tags, generate_snippet};

        let raw = "<p>This is a &amp; test with <b>bold</b> and <a href='https://example.com'>link</a> text.</p>";
        let cleaned = clean_html_tags(raw);
        assert_eq!(cleaned, "This is a & test with bold and link text.");

        let snippet = generate_snippet(raw, 20);
        assert!(snippet.chars().count() <= 22);
    }

    #[test]
    fn test_configurable_keybindings_matching() {
        let slash_key = KeyEvent {
            code: KeyCode::Char('/'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(key_matches(&slash_key, "/"));
        assert!(key_matches(&slash_key, "?, /"));

        let ctrl_f = KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(key_matches(&ctrl_f, "ctrl+f"));
        assert!(key_matches(&ctrl_f, "ctrl+f, ctrl+s"));

        let shift_m = KeyEvent {
            code: KeyCode::Char('M'),
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(key_matches(&shift_m, "M"));

        let j_key = KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(key_matches(&j_key, "j, down"));

        let down_key = KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        };
        assert!(key_matches(&down_key, "j, down"));
    }

    #[test]
    fn test_app_config_keybindings_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.keybindings.toggle_config, "/");
        assert!(config.keybindings.quit.contains("q"));
        assert!(config.keybindings.search.contains("ctrl+f"));
    }

    #[tokio::test]
    async fn test_mouse_events_handling() {
        use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
        use ratarss::model::ActivePane;

        let rt = tokio::runtime::Handle::current();
        let mut app = ratarss::App::new(rt).expect("app new failed");

        // Click on Sidebar (column 5, row 3)
        let click_sidebar = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 3,
            modifiers: KeyModifiers::NONE,
        };
        app.handle_mouse_event(click_sidebar, 100, 30);
        assert_eq!(app.active_pane, ActivePane::Sidebar);

        // Click on Article List (column 35, row 5)
        let click_articles = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 35,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        app.handle_mouse_event(click_articles, 100, 30);
        assert_eq!(app.active_pane, ActivePane::ArticleList);

        // Click on Reader View (column 80, row 10)
        let click_reader = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 80,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        app.handle_mouse_event(click_reader, 100, 30);
        assert_eq!(app.active_pane, ActivePane::Reader);

        // Scroll in reader
        let scroll_down = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 80,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        let prev_offset = app.reader_scroll_offset;
        app.handle_mouse_event(scroll_down, 100, 30);
        assert!(app.reader_scroll_offset >= prev_offset);
    }

    #[test]
    fn test_zstd_compression_and_decompression() {
        use ratarss::storage::{compress_text, decompress_text, Database};
        use ratarss::model::CurrentFilter;

        let sample_html = "<article><h1>Very Large Article Content</h1><p>".repeat(50);
        let compressed = compress_text(&sample_html);
        assert!(compressed.starts_with(b"ZSTD"));
        assert!(compressed.len() < sample_html.len(), "Zstd should significantly reduce text size");

        let decompressed = decompress_text(&compressed).expect("decompress failed");
        assert_eq!(decompressed, sample_html);

        // Verify storage roundtrip with zstd in SQLite
        let db = Database::in_memory().expect("db in memory");
        let stats = db.get_db_stats().expect("stats failed");
        assert_eq!(stats.compression, "Zstandard (zstd level 3)");
        assert!(stats.wal_mode);

        let filter = CurrentFilter::Smart(ratarss::model::SmartFeedKind::AllArticles);
        let articles = db.get_articles_by_filter(&filter).expect("get articles failed");
        assert!(!articles.is_empty());
        assert!(articles[0].content.is_some());
    }

    /// A database written before compression holds `summary`/`content` as TEXT.
    /// Every smart feed must still list those articles: the sidebar counts them
    /// with a SQL aggregate that never looks at the column's type, so if the
    /// list query cannot read them the two disagree and the pane goes blank
    /// while the count says thousands.
    #[test]
    fn test_legacy_uncompressed_text_rows_still_list() {
        use ratarss::model::{CurrentFilter, SmartFeedKind};
        use ratarss::storage::Database;
        use rusqlite::{params, Connection};

        let db = Database::in_memory().expect("db in memory");

        let conn = Connection::open(db.get_path()).expect("open");
        conn.execute(
            "INSERT INTO articles (id, feed_id, feed_title, title, author, summary, content,
                                   url, published, read, starred, created_at)
             VALUES ('legacy-1', 'bloomberg-markets', 'Legacy Feed', 'Written before zstd', NULL,
                     ?1, ?2, 'https://example.com/legacy', ?3, 0, 0, ?3)",
            // Dated now, so the `Today` assertion below is about the column
            // type rather than about the date falling outside the window.
            params![
                "a plain TEXT summary",
                "a plain TEXT body",
                Utc::now().timestamp()
            ],
        )
        .expect("insert legacy row");

        assert_eq!(
            "text",
            conn.query_row(
                "SELECT typeof(summary) FROM articles WHERE id = 'legacy-1'",
                [],
                |r| r.get::<_, String>(0)
            )
            .expect("typeof"),
            "the fixture must actually be TEXT or it proves nothing"
        );
        drop(conn);

        for kind in [
            SmartFeedKind::AllArticles,
            SmartFeedKind::AllUnread,
            SmartFeedKind::Today,
        ] {
            let listed = db
                .get_articles_by_filter(&CurrentFilter::Smart(kind))
                .unwrap_or_else(|e| panic!("{kind:?} failed to list at all: {e}"));
            assert!(
                listed.iter().any(|a| a.id == "legacy-1"),
                "{kind:?} dropped the legacy row"
            );
        }

        let listed = db
            .get_articles_by_filter(&CurrentFilter::Smart(SmartFeedKind::AllArticles))
            .expect("all articles");
        let legacy = listed.iter().find(|a| a.id == "legacy-1").unwrap();
        assert_eq!(legacy.summary.as_deref(), Some("a plain TEXT summary"));
        assert_eq!(legacy.content.as_deref(), Some("a plain TEXT body"));
    }
}
