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

        let formatted = render_article_to_text(&article, None, &theme, 80);
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
        assert!(config.keybindings.search.contains("ctrl+a"));
        assert!(config.keybindings.search_feeds.contains("ctrl+f"));
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
        // The list query deliberately leaves bodies behind; they are fetched
        // one at a time for the reader.
        assert!(articles[0].content.is_none());
        assert!(db.get_article_body(&articles[0].id).is_some());
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
        assert_eq!(
            db.get_article_body("legacy-1").as_deref(),
            Some("a plain TEXT body"),
            "the reader must still decode a legacy TEXT body"
        );
    }
}

/// Rendering smoke tests.
///
/// The chrome rework changed pane geometry (no top bar, configurable padding)
/// and replaced per-cell background fills with whole-rect ones. Both are
/// arithmetic on `Rect`s, where an off-by-one panics inside the buffer rather
/// than merely looking wrong, so draw the panes at awkward sizes.
#[cfg(test)]
mod render_tests {
    use chrono::Utc;
    use ratarss::model::{ActivePane, Article, ArticleSlice, SidebarItem, SmartFeedKind};
    use ratarss::theme::Theme;
    use ratarss::ui::article_list::ArticleListView;
    use ratarss::ui::sidebar::SidebarView;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    fn article(n: usize) -> Article {
        Article {
            id: format!("a{n}"),
            feed_id: "f".to_string(),
            feed_title: "A Rather Long Feed Title".to_string(),
            title: "An article title that is long enough to need truncating".to_string(),
            author: None,
            summary: Some("Summary text that also runs past the pane edge".to_string()),
            content: None,
            url: "https://example.com".to_string(),
            published: Some(Utc::now()),
            read: n % 2 == 0,
            starred: n % 3 == 0,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn panes_render_at_any_width_and_padding() {
        let theme = Theme::ratarss_dark();
        let articles: Vec<Article> = (0..12).map(article).collect();
        let items = vec![
            SidebarItem::SmartHeader,
            SidebarItem::Smart(SmartFeedKind::Today, 12808),
            SidebarItem::FolderHeader {
                name: "Substack".to_string(),
                is_expanded: true,
                unread_count: 457,
                feed_count: 9,
            },
            SidebarItem::Feed {
                feed_id: "f".to_string(),
                title: "A feed with a name too long for a narrow pane".to_string(),
                folder: Some("Substack".to_string()),
                unread_count: 20,
                has_error: true,
            },
        ];

        for width in [8u16, 12, 20, 40, 120] {
            for padding in [0u16, 1, 3, 6] {
                for show_icons in [true, false] {
                    let area = Rect::new(0, 0, width, 14);
                    let mut buf = Buffer::empty(area);

                    SidebarView {
                        items: &items,
                        selected_index: 1,
                        active_pane: ActivePane::Sidebar,
                        theme: &theme,
                        scroll_offset: 0,
                        show_icons,
                        padding,
                        search_query: "fee",
                        is_searching: padding % 2 == 0,
                    }
                    .render(area, &mut buf);

                    ArticleListView {
                        articles: ArticleSlice {
                            all: &articles,
                            filtered: None,
                        },
                        selected_index: 3,
                        active_pane: ActivePane::ArticleList,
                        theme: &theme,
                        header_title: "All Articles",
                        unread_count: 12808,
                        search_query: "query",
                        is_searching: true,
                        scroll_offset: 0,
                        show_icons,
                        padding,
                        spacing: padding % 4,
                    }
                    .render(area, &mut buf);
                }
            }
        }
    }

    /// A `<` immediately followed by a multi-byte character used to panic the
    /// block-tag normalizer, which compared a byte-length slice of the input
    /// against each tag. Real feeds are full of curly quotes right after tags.
    #[test]
    fn html_with_multibyte_characters_after_tags_renders() {
        let theme = Theme::ratarss_dark();
        let mut a = article(1);
        a.title = "Curly ’quotes’".to_string();

        for html in [
            "<p>It’s fine</p>",
            "<’>",
            "<b>—</b><br/>naïve café ’’’",
            "<blockquote>“Quoted”</blockquote><li>é</li>",
        ] {
            let formatted = ratarss::reader::render_article_to_text(&a, Some(html), &theme, 40);
            assert!(formatted.total_lines > 0, "{html:?} produced nothing");
        }
    }

    /// Links in the body must survive wrapping as their own spans, and the
    /// recorded hit must point at the columns the anchor text was actually
    /// drawn in — that mapping is what makes them clickable.
    #[test]
    fn body_links_are_styled_and_hit_testable() {
        let theme = Theme::ratarss_dark();
        let a = article(1);
        let html = "<p>Go read <a href=\"https://example.com/story\">this fine story</a> today \
                    and then <a href=\"https://example.com/other\">another</a>.</p>";

        let formatted = ratarss::reader::render_article_to_text(&a, Some(html), &theme, 60);

        assert!(formatted.links.iter().any(|l| l == "https://example.com/story"));
        assert!(formatted.links.iter().any(|l| l == "https://example.com/other"));

        // Every hit must land on the text it claims to cover.
        assert!(!formatted.hits.is_empty());
        for hit in &formatted.hits {
            let url = formatted.links[hit.link].clone();
            let found = formatted
                .link_at(hit.line, hit.start_col)
                .expect("hit start must resolve to its link");
            assert_eq!(found, url);
            assert_eq!(formatted.link_at(hit.line, hit.end_col - 1).unwrap(), url);
            // The columns must be inside the line that was drawn.
            let line_width = formatted.lines[hit.line].width() as u16;
            assert!(
                hit.end_col <= line_width,
                "hit {:?} runs past its {line_width}-wide line",
                hit
            );
        }

        // A column outside every hit is not a link.
        let body_hit = formatted.hits.last().unwrap();
        assert!(formatted.link_at(body_hit.line, body_hit.end_col).is_none());
    }

    /// The header URL is clickable too, and a link split across a wrap still
    /// resolves on both of its lines.
    #[test]
    fn header_url_and_wrapped_links_resolve() {
        let theme = Theme::ratarss_dark();
        let a = article(1);
        let long = "<p><a href=\"https://example.com/x\">a very long anchor label that will \
                    certainly need to wrap across more than one line of the pane</a></p>";

        let formatted = ratarss::reader::render_article_to_text(&a, Some(long), &theme, 40);

        // article(1).url is https://example.com
        assert!(formatted.links.iter().any(|l| l == "https://example.com"));
        let header = formatted.hits.first().unwrap();
        assert_eq!(
            formatted.link_at(header.line, header.start_col).unwrap(),
            "https://example.com"
        );

        let wrapped: Vec<_> = formatted
            .hits
            .iter()
            .filter(|h| formatted.links[h.link] == "https://example.com/x")
            .collect();
        assert!(wrapped.len() > 1, "the long anchor should span several lines");
    }

    /// The list must run to the bottom edge of its pane. Card height rarely
    /// divides the available rows, and skipping the card that does not fully
    /// fit left a visible dead band above the border; the last card is drawn
    /// clipped instead.
    #[test]
    fn list_fills_to_the_bottom_edge() {
        let theme = Theme::ratarss_dark();
        let articles: Vec<Article> = (0..40).map(article).collect();

        for height in 8u16..24 {
            for spacing in [0u16, 1, 2] {
                let area = Rect::new(0, 0, 50, height);
                let mut buf = Buffer::empty(area);
                ArticleListView {
                    articles: ArticleSlice { all: &articles, filtered: None },
                    selected_index: 0,
                    active_pane: ActivePane::ArticleList,
                    theme: &theme,
                    header_title: "All",
                    unread_count: 3,
                    search_query: "",
                    is_searching: false,
                    scroll_offset: 0,
                    show_icons: true,
                    padding: 1,
                    spacing,
                }
                .render(area, &mut buf);

                // Count blank rows above the bottom border. Anything beyond a
                // single inter-card gap is the dead band this fixes.
                let mut blank = 0u16;
                for y in (1..height - 1).rev() {
                    let row: String = (1..area.width - 1)
                        .map(|x| buf[(x, y)].symbol().to_string())
                        .collect();
                    if row.trim().is_empty() {
                        blank += 1;
                    } else {
                        break;
                    }
                }
                assert!(
                    blank <= spacing,
                    "height {height} spacing {spacing}: {blank} dead rows at the bottom"
                );
            }
        }
    }

    /// Half-row spacing cannot exist in a cell grid, so it is approximated by
    /// alternating gaps. The layout must average out, stay monotonic, and agree
    /// with the click hit-test that maps a row back to a card.
    #[test]
    fn half_row_spacing_distributes_and_round_trips() {
        use ratarss::ui::article_list::{card_at_row, card_top, spacing_label, CARD_HEIGHT};

        assert_eq!(spacing_label(0), "0");
        assert_eq!(spacing_label(1), "0.5");
        assert_eq!(spacing_label(3), "1.5");

        // Half-row spacing: gaps alternate 0, 1, 0, 1 ...
        let gaps: Vec<u16> = (0..6)
            .map(|k| card_top(k + 1, 1) - card_top(k, 1) - CARD_HEIGHT)
            .collect();
        assert_eq!(gaps, vec![0, 1, 0, 1, 0, 1]);

        // Flush and whole-row spacing stay exact.
        assert_eq!(card_top(4, 0), 12);
        assert_eq!(card_top(4, 2), 16);

        for halves in 0..=6u16 {
            // Monotonic, and every row inside a card maps back to that card.
            for k in 0..12usize {
                let top = card_top(k, halves);
                assert!(k == 0 || top > card_top(k - 1, halves));
                for row in top..top + CARD_HEIGHT {
                    assert_eq!(
                        card_at_row(row, halves),
                        k,
                        "halves {halves}: row {row} should hit card {k}"
                    );
                }
            }
        }
    }

    /// `ctrl+f` changed meaning from article search to feed search. A config
    /// written before that still says `search = "ctrl+f, ctrl+s"`, which would
    /// leave the user with no article-search binding and two things fighting
    /// over ctrl+f, so the untouched default is migrated on load — and a
    /// customised binding is not.
    #[test]
    fn old_search_binding_migrates_but_custom_one_survives() {
        use ratarss::config::AppConfig;

        let old = r#"
theme = "Nord"
sidebar_ratio = 22
article_list_ratio = 33
reader_ratio = 45
refresh_interval_minutes = 15
auto_refresh_on_startup = true
mark_read_on_open = true
max_articles_per_feed = 200
show_icons = true
wrap_article_text = true
"#;

        // Old default: migrated, and feed search gets the new default.
        let mut cfg: AppConfig =
            toml::from_str(&format!("{old}\n[keybindings]\nsearch = \"ctrl+f, ctrl+s\"\n"))
                .expect("old config must still parse");
        assert_eq!(cfg.keybindings.search_feeds, "ctrl+f");
        cfg.keybindings.migrate();
        assert_eq!(cfg.keybindings.search, "ctrl+a");

        // Custom binding: left alone.
        let mut custom: AppConfig =
            toml::from_str(&format!("{old}\n[keybindings]\nsearch = \"ctrl+k\"\n"))
                .expect("custom config must parse");
        custom.keybindings.migrate();
        assert_eq!(custom.keybindings.search, "ctrl+k");
    }

    /// Fuzzy search is only useful if the ranking is right: an acronym should
    /// find the multi-word title, a prefix should beat a mid-word coincidence,
    /// and out-of-order characters should not match at all.
    /// Drive the real key handler: ctrl+f must open the sidebar's feed filter
    /// and ctrl+a the article one. Assertions hold for any database contents —
    /// the smart views are always present, and no feed matches gibberish.
    #[tokio::test]
    async fn search_bindings_enter_the_right_mode() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
        use ratarss::model::ActivePane;

        fn key(c: char, mods: KeyModifiers) -> KeyEvent {
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: mods,
                kind: KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            }
        }

        let rt = tokio::runtime::Handle::current();
        let mut app = ratarss::App::new(rt).expect("app new failed");
        let sidebar_len = app.sidebar_items.len();
        assert!(sidebar_len > 0, "smart views should always be listed");

        // ctrl+f -> feed search, focused on the sidebar.
        app.handle_key_event(key('f', KeyModifiers::CONTROL));
        assert!(app.is_searching_feeds);
        assert_eq!(app.active_pane, ActivePane::Sidebar);

        // Typing filters the sidebar; gibberish matches nothing.
        for c in "zqxjv".chars() {
            app.handle_key_event(key(c, KeyModifiers::NONE));
        }
        assert_eq!(app.feed_search_query, "zqxjv");
        assert!(app.sidebar_items.is_empty(), "no feed should fuzzy-match gibberish");

        // Esc restores the normal sidebar.
        app.handle_key_event(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
        assert!(!app.is_searching_feeds);
        assert!(app.feed_search_query.is_empty());
        assert_eq!(app.sidebar_items.len(), sidebar_len);

        // ctrl+a -> article search, focused on the list.
        app.handle_key_event(key('a', KeyModifiers::CONTROL));
        assert!(app.is_searching);
        assert_eq!(app.active_pane, ActivePane::ArticleList);
    }

    #[test]
    fn fuzzy_ranks_the_way_a_reader_expects() {
        use ratarss::fuzzy::{score, score_any};

        // Subsequence, case-insensitive.
        assert!(score("Bloomberg Markets", "bm").is_some());
        assert!(score("Bloomberg Markets", "blmkt").is_some());
        // Out of order does not match. ("mb" would match Bloo*m*_b_erg, which is
        // correct subsequence behaviour, so pick letters that truly reverse.)
        assert!(score("Hacker News", "nh").is_none());
        // Absent character does not match.
        assert!(score("Hacker News", "hz").is_none());
        // Empty needle matches everything; empty haystack matches nothing.
        assert_eq!(score("anything", ""), Some(0));
        assert!(score("", "a").is_none());

        // Word-start acronym beats an accidental mid-word subsequence.
        let acronym = score("Bloomberg Markets", "bm").unwrap();
        let midword = score("Abbreviated Plumber", "bm").unwrap();
        assert!(acronym > midword, "acronym {acronym} should beat {midword}");

        // A prefix match beats the same letters buried later.
        let prefix = score("Rust Blog", "rust").unwrap();
        let buried = score("The Trust Fall", "rust").unwrap();
        assert!(prefix > buried, "prefix {prefix} should beat {buried}");

        // Consecutive runs beat scattered characters.
        let tight = score("Hacker News", "hacker").unwrap();
        let loose = score("Have A Chance To Kill Every Rabbit", "hacker").unwrap();
        assert!(tight > loose, "tight {tight} should beat loose {loose}");

        // score_any takes the best field.
        let best = score_any(["nothing here", "Hacker News"], "hn").unwrap();
        assert_eq!(best, score("Hacker News", "hn").unwrap());
        assert!(score_any(["alpha", "beta"], "zz").is_none());
    }

    #[test]
    fn article_slice_indexes_through_search_matches() {
        let articles: Vec<Article> = (0..5).map(article).collect();
        let filtered = [1u32, 4];
        let slice = ArticleSlice {
            all: &articles,
            filtered: Some(&filtered),
        };

        assert_eq!(slice.len(), 2);
        assert_eq!(slice.get(0).unwrap().id, "a1");
        assert_eq!(slice.get(1).unwrap().id, "a4");
        assert!(slice.get(2).is_none());
    }
}
