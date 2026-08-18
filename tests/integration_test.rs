#[cfg(test)]
mod tests {
    use ratarss::opml::{export_opml, parse_opml_str};
    use ratarss::model::Feed;
    use ratarss::theme::Theme;
    use ratarss::reader::render_article_to_text;
    use ratarss::model::Article;
    use chrono::Utc;

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
        assert!(!presets.is_empty());
        let dark = Theme::by_name("NetNewsWire Dark");
        assert_eq!(dark.config.name, "NetNewsWire Dark");
        let tokyo = Theme::by_name("Tokyo Night");
        assert_eq!(tokyo.config.name, "Tokyo Night");
    }

    #[test]
    fn test_reader_rendering() {
        let theme = Theme::netnewswire_dark();
        let article = Article {
            id: "test-1".to_string(),
            feed_id: "test-feed".to_string(),
            feed_title: "Tech News".to_string(),
            title: "Rust 2026 Edition Released".to_string(),
            author: Some("Ferris".to_string()),
            summary: Some("Summary preview".to_string()),
            content: Some("<p>Hello <strong>World</strong>!</p><blockquote>Quoted wisdom</blockquote>".to_string()),
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
        use ratarss::storage::Database;
        use ratarss::model::{CurrentFilter, SmartFeedKind};

        let db = Database::in_memory().expect("in memory db failed");
        let feeds = db.get_all_feeds().expect("get feeds failed");
        assert!(!feeds.is_empty(), "Database should seed with default feeds");

        let counts = db.get_unread_counts().expect("counts failed");
        assert!(counts.all_articles > 0);

        let today_articles = db.get_articles_by_filter(&CurrentFilter::Smart(SmartFeedKind::Today)).expect("today failed");
        assert!(!today_articles.is_empty());

        let first_id = &today_articles[0].id;
        db.set_article_read(first_id, true).expect("mark read failed");
        let updated_counts = db.get_unread_counts().expect("updated counts failed");
        assert_eq!(updated_counts.all_unread, counts.all_unread.saturating_sub(1));

        db.set_article_starred(first_id, true).expect("star failed");
        let starred = db.get_articles_by_filter(&CurrentFilter::Smart(SmartFeedKind::Starred)).expect("starred query failed");
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
}
