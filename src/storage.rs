use crate::config::AppConfig;
use crate::model::{Article, CurrentFilter, Feed, SmartFeedKind};
use crate::sample_data;
use chrono::{DateTime, Utc};
use rusqlite::types::ValueRef;
use rusqlite::{params, Connection, OptionalExtension, Result, Row};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

/// Magic header prefix for Zstd compressed blobs in SQLite
const ZSTD_MAGIC: &[u8; 4] = b"ZSTD";

/// Compress text using Zstandard (level 3 for optimal ratio and gigabytes/sec decompression)
pub fn compress_text(text: &str) -> Vec<u8> {
    if text.is_empty() {
        return Vec::new();
    }
    match zstd::encode_all(text.as_bytes(), 3) {
        Ok(compressed) => {
            let mut result = Vec::with_capacity(4 + compressed.len());
            result.extend_from_slice(ZSTD_MAGIC);
            result.extend_from_slice(&compressed);
            result
        }
        Err(_) => text.as_bytes().to_vec(),
    }
}

/// Decompress bytes into UTF-8 String (handles both Zstd-compressed blobs and raw UTF-8 fallback)
pub fn decompress_text(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }
    if bytes.starts_with(ZSTD_MAGIC) && bytes.len() >= 4 {
        match zstd::decode_all(&bytes[4..]) {
            Ok(decompressed) => String::from_utf8(decompressed).ok(),
            Err(_) => String::from_utf8(bytes.to_vec()).ok(),
        }
    } else {
        // Fallback for raw legacy uncompressed text
        match zstd::decode_all(bytes) {
            Ok(decompressed) => String::from_utf8(decompressed).ok(),
            Err(_) => String::from_utf8(bytes.to_vec()).ok(),
        }
    }
}

/// Read a `summary`/`content` column that may hold either a compressed BLOB or
/// legacy TEXT.
///
/// Both storage forms are live in the same table: rows written before
/// compression are TEXT, rows written after are `ZSTD`-prefixed BLOBs, and
/// `CREATE TABLE IF NOT EXISTS` never rewrote the old ones. Asking rusqlite for
/// a `Vec<u8>` fails on TEXT with `InvalidColumnType`, and because that error
/// aborts the whole row loop, a single legacy row empties an entire article
/// list. So dispatch on what SQLite actually stored rather than on what the
/// column was declared as.
fn read_body_column(row: &Row, idx: usize) -> Result<Option<String>> {
    match row.get_ref(idx)? {
        ValueRef::Null => Ok(None),
        ValueRef::Blob(bytes) => Ok(decompress_text(bytes)),
        ValueRef::Text(bytes) => Ok(String::from_utf8(bytes.to_vec()).ok()),
        // An integer or float here means something else wrote the column; keep
        // the row rather than losing the article over its body.
        _ => Ok(None),
    }
}

/// Longest plain-text preview kept per article for the list pane.
const SNIPPET_CHARS: usize = 220;

/// Build the list preview: tags stripped, entities decoded, whitespace
/// collapsed, truncated.
///
/// Stored uncompressed so opening a view never has to decompress anything.
pub fn make_snippet(summary: Option<&str>, content: Option<&str>) -> Option<String> {
    let source = summary
        .filter(|s| !s.trim().is_empty())
        .or(content.filter(|s| !s.trim().is_empty()))?;

    let mut out = String::with_capacity(SNIPPET_CHARS);
    let mut in_tag = false;
    let mut pending_space = false;
    for ch in source.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                pending_space = true;
            }
            _ if in_tag => {}
            c if c.is_whitespace() => pending_space = !out.is_empty(),
            c => {
                if pending_space {
                    out.push(' ');
                    pending_space = false;
                }
                out.push(c);
                if out.chars().count() >= SNIPPET_CHARS {
                    break;
                }
            }
        }
    }

    let decoded = html_escape::decode_html_entities(&out).into_owned();
    let trimmed = decoded.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// One SQLite connection, shared by every clone of `Database` and by the
/// background fetch tasks.
///
/// Opening a connection costs a file open plus the six `PRAGMA` round trips
/// below, and the app used to pay that on every single read and write —
/// including once per folder while rebuilding the sidebar. Holding one
/// connection behind a mutex keeps `Database: Send + Sync + Clone` (what the
/// tokio tasks need) while letting rusqlite's statement cache actually survive
/// between calls.
#[derive(Clone)]
pub struct Database {
    db_path: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, Default, Clone)]
pub struct UnreadCounts {
    pub today: usize,
    pub all_unread: usize,
    pub starred: usize,
    pub all_articles: usize,
    pub feed_counts: HashMap<String, (usize, usize)>, // feed_id -> (unread, total)
    pub folder_counts: HashMap<String, (usize, usize)>, // folder_name -> (unread, total)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub db_path: PathBuf,
    pub file_size_bytes: u64,
    pub total_articles: usize,
    pub total_feeds: usize,
    pub compression: String,
    pub wal_mode: bool,
}

#[allow(dead_code)]
impl Database {
    pub fn new() -> Result<Self> {
        let data_dir = AppConfig::get_data_dir();
        fs::create_dir_all(&data_dir).ok();
        let db_path = data_dir.join("ratarss.db");
        let db = Self::open(db_path)?;
        db.init_schema()?;
        let _ = db.compress_uncompressed_articles();
        let _ = db.backfill_snippets();
        Ok(db)
    }

    pub fn in_memory() -> Result<Self> {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("ratarss_test_{}.db", uuid::Uuid::new_v4()));
        let db = Self::open(db_path)?;
        db.init_schema()?;
        Ok(db)
    }

    fn open(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 10000;
             PRAGMA foreign_keys = ON;
             PRAGMA cache_size = -8000;
             PRAGMA temp_store = MEMORY;",
        )?;
        // Statements are re-run constantly (list reloads, read/star toggles);
        // caching them skips re-parsing the SQL each time.
        conn.set_prepared_statement_cache_capacity(32);
        Ok(Self {
            db_path,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// A poisoned lock only means some other thread panicked mid-query; the
    /// connection itself is still usable, so recover rather than take the app
    /// down with it.
    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn();

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS feeds (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                site_url TEXT,
                folder TEXT,
                custom_icon TEXT,
                last_updated INTEGER,
                error TEXT
            );

            CREATE TABLE IF NOT EXISTS articles (
                id TEXT PRIMARY KEY,
                feed_id TEXT NOT NULL,
                feed_title TEXT NOT NULL,
                title TEXT NOT NULL,
                author TEXT,
                summary BLOB,
                content BLOB,
                url TEXT NOT NULL,
                published INTEGER,
                read INTEGER NOT NULL DEFAULT 0,
                starred INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS folders (
                name TEXT PRIMARY KEY,
                is_expanded INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_articles_feed_id ON articles(feed_id);
            CREATE INDEX IF NOT EXISTS idx_articles_feed_read ON articles(feed_id, read);
            CREATE INDEX IF NOT EXISTS idx_articles_read ON articles(read);
            CREATE INDEX IF NOT EXISTS idx_articles_starred ON articles(starred);
            CREATE INDEX IF NOT EXISTS idx_feeds_folder ON feeds(folder);
            "
        )?;

        // Columns added after the original schema. `CREATE TABLE IF NOT EXISTS`
        // never revisits an existing table, so add them explicitly.
        //
        // `sort_ts` is the value every view sorts by. Sorting on
        // COALESCE(published, created_at) is not something an index can serve,
        // so every list build ran a temp B-tree over the whole table; with the
        // value stored, the indexes below deliver rows already in order.
        //
        // `snippet` is the plain-text preview the list draws. It exists so the
        // list query touches no compressed blob at all — decompressing every
        // article's summary just to show one truncated line was the bulk of the
        // cost of opening a large view.
        let existing: Vec<String> = {
            let mut stmt = conn.prepare("PRAGMA table_info(articles)")?;
            let names = stmt.query_map([], |r| r.get::<_, String>(1))?;
            names.flatten().collect()
        };
        if !existing.iter().any(|c| c == "sort_ts") {
            conn.execute_batch("ALTER TABLE articles ADD COLUMN sort_ts INTEGER;")?;
        }
        if !existing.iter().any(|c| c == "snippet") {
            conn.execute_batch("ALTER TABLE articles ADD COLUMN snippet TEXT;")?;
        }

        conn.execute_batch(
            "
            CREATE INDEX IF NOT EXISTS idx_articles_sort ON articles(sort_ts DESC);
            CREATE INDEX IF NOT EXISTS idx_articles_read_sort ON articles(read, sort_ts DESC);
            CREATE INDEX IF NOT EXISTS idx_articles_starred_sort ON articles(starred, sort_ts DESC);
            CREATE INDEX IF NOT EXISTS idx_articles_feed_sort ON articles(feed_id, sort_ts DESC);
            ",
        )?;

        // Cheap and idempotent: fills the sort key for rows written before it existed.
        conn.execute(
            "UPDATE articles SET sort_ts = COALESCE(published, created_at) WHERE sort_ts IS NULL",
            [],
        )?;

        // Every view now filters and orders on `sort_ts`, so a row missing it
        // is a row that silently vanishes from Today. Rather than trusting each
        // writer to remember, hold the invariant in the schema: anything
        // inserted or repointed without it gets it derived on the spot.
        //
        // The snippet trigger only covers legacy TEXT summaries, which SQL can
        // read; compressed ones need `backfill_snippets`.
        conn.execute_batch(
            "
            CREATE TRIGGER IF NOT EXISTS articles_sort_ts_insert
            AFTER INSERT ON articles WHEN new.sort_ts IS NULL
            BEGIN
                UPDATE articles SET sort_ts = COALESCE(new.published, new.created_at)
                WHERE id = new.id;
            END;

            CREATE TRIGGER IF NOT EXISTS articles_sort_ts_update
            AFTER UPDATE OF published, created_at ON articles
            BEGIN
                UPDATE articles SET sort_ts = COALESCE(new.published, new.created_at)
                WHERE id = new.id;
            END;

            CREATE TRIGGER IF NOT EXISTS articles_snippet_insert
            AFTER INSERT ON articles
            WHEN new.snippet IS NULL AND typeof(new.summary) = 'text'
            BEGIN
                UPDATE articles SET snippet = substr(new.summary, 1, 220) WHERE id = new.id;
            END;
            ",
        )?;

        // If feeds table is empty, seed with default curated feeds and sample articles
        let feed_count: i64 = conn.query_row("SELECT COUNT(*) FROM feeds", [], |r| r.get(0))?;
        if feed_count == 0 {
            for feed in sample_data::default_feeds() {
                conn.execute(
                    "INSERT OR IGNORE INTO feeds (id, title, url, site_url, folder, custom_icon, last_updated, error)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        feed.id,
                        feed.title,
                        feed.url,
                        feed.site_url,
                        feed.folder,
                        feed.custom_icon,
                        feed.last_updated.map(|d| d.timestamp()),
                        feed.error
                    ],
                )?;
            }

            for article in sample_data::sample_articles() {
                let compressed_summary = article.summary.as_deref().map(compress_text);
                let compressed_content = article.content.as_deref().map(compress_text);

                let snippet = make_snippet(article.summary.as_deref(), article.content.as_deref());
                let sort_ts = article.published.unwrap_or(article.created_at).timestamp();

                conn.execute(
                    "INSERT OR IGNORE INTO articles (id, feed_id, feed_title, title, author, summary, content, url, published, read, starred, created_at, sort_ts, snippet)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                    params![
                        article.id,
                        article.feed_id,
                        article.feed_title,
                        article.title,
                        article.author,
                        compressed_summary,
                        compressed_content,
                        article.url,
                        article.published.map(|d| d.timestamp()),
                        if article.read { 1 } else { 0 },
                        if article.starred { 1 } else { 0 },
                        article.created_at.timestamp(),
                        sort_ts,
                        snippet,
                    ],
                )?;
            }
        }

        Ok(())
    }

    /// Automatically migrates and compresses any legacy uncompressed articles using Zstandard.
    ///
    /// The scan behind this is a full table scan that no index can serve, so it
    /// is recorded as done in `settings` once it completes: on a large archive
    /// re-scanning every article on every launch is the single slowest thing
    /// startup could do, and nothing writes uncompressed bodies any more.
    pub fn compress_uncompressed_articles(&self) -> Result<usize> {
        if self.get_setting("compression_migrated").as_deref() == Some("1") {
            return Ok(0);
        }

        let mut conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, summary, content FROM articles 
             WHERE (summary IS NOT NULL AND substr(summary, 1, 4) != X'5A535444') 
                OR (content IS NOT NULL AND substr(content, 1, 4) != X'5A535444')"
        )?;

        let rows = stmt.query_map([], |r| {
            let id: String = r.get(0)?;
            Ok((id, read_body_column(r, 1)?, read_body_column(r, 2)?))
        })?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }

        if items.is_empty() {
            drop(stmt);
            drop(conn);
            self.set_setting("compression_migrated", "1")?;
            return Ok(0);
        }

        drop(stmt);

        let tx = conn.transaction()?;
        let mut updated = 0;
        for (id, sum_text, cont_text) in items {
            let new_sum = sum_text.as_deref().map(compress_text);
            let new_cont = cont_text.as_deref().map(compress_text);
            tx.execute(
                "UPDATE articles SET summary = ?1, content = ?2 WHERE id = ?3",
                params![new_sum, new_cont, id],
            )?;
            updated += 1;
        }
        tx.commit()?;
        drop(conn);
        self.set_setting("compression_migrated", "1")?;
        Ok(updated)
    }

    /// Fill `snippet` for rows written before the column existed.
    ///
    /// This is the only pass that decompresses in bulk, and it runs once: after
    /// it, opening a view reads plain text straight out of the index-ordered
    /// scan. Guarded by a settings flag like the compression migration.
    pub fn backfill_snippets(&self) -> Result<usize> {
        if self.get_setting("snippets_backfilled").as_deref() == Some("1") {
            return Ok(0);
        }

        let mut conn = self.conn();
        let mut items: Vec<(String, Option<String>)> = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT id, summary, content FROM articles WHERE snippet IS NULL",
            )?;
            let rows = stmt.query_map([], |r| {
                let id: String = r.get(0)?;
                let summary = read_body_column(r, 1)?;
                let content = read_body_column(r, 2)?;
                Ok((id, make_snippet(summary.as_deref(), content.as_deref())))
            })?;
            for row in rows {
                items.push(row?);
            }
        }

        let updated = items.len();
        if updated > 0 {
            let tx = conn.transaction()?;
            {
                let mut stmt =
                    tx.prepare_cached("UPDATE articles SET snippet = ?1 WHERE id = ?2")?;
                for (id, snippet) in items {
                    // An empty string, not NULL, so the row is not rescanned.
                    stmt.execute(params![snippet.unwrap_or_default(), id])?;
                }
            }
            tx.commit()?;
        }

        drop(conn);
        self.set_setting("snippets_backfilled", "1")?;
        Ok(updated)
    }

    fn get_setting(&self, key: &str) -> Option<String> {
        let conn = self.conn();
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten()
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_db_stats(&self) -> Result<DatabaseStats> {
        let conn = self.conn();
        let total_articles: usize = conn.query_row("SELECT COUNT(*) FROM articles", [], |r| r.get(0)).unwrap_or(0);
        let total_feeds: usize = conn.query_row("SELECT COUNT(*) FROM feeds", [], |r| r.get(0)).unwrap_or(0);
        let file_size_bytes = fs::metadata(&self.db_path).map(|m| m.len()).unwrap_or(0);

        Ok(DatabaseStats {
            db_path: self.db_path.clone(),
            file_size_bytes,
            total_articles,
            total_feeds,
            compression: "Zstandard (zstd level 3)".to_string(),
            wal_mode: true,
        })
    }

    pub fn get_all_feeds(&self) -> Result<Vec<Feed>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, title, url, site_url, folder, custom_icon, last_updated, error FROM feeds ORDER BY folder ASC, title ASC"
        )?;
        
        let feed_rows = stmt.query_map([], |row| {
            let last_updated_ts: Option<i64> = row.get(6)?;
            let last_updated = last_updated_ts.and_then(|ts| DateTime::from_timestamp(ts, 0));
            
            Ok(Feed {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                site_url: row.get(3)?,
                folder: row.get(4)?,
                custom_icon: row.get(5)?,
                last_updated,
                error: row.get(7)?,
                unread_count: 0,
                total_count: 0,
            })
        })?;

        let mut feeds = Vec::new();
        for f in feed_rows {
            feeds.push(f?);
        }
        Ok(feeds)
    }

    pub fn add_or_update_feed(&self, feed: &Feed) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO feeds (id, title, url, site_url, folder, custom_icon, last_updated, error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(url) DO UPDATE SET
                title = excluded.title,
                site_url = COALESCE(excluded.site_url, feeds.site_url),
                folder = COALESCE(excluded.folder, feeds.folder),
                last_updated = excluded.last_updated,
                error = excluded.error",
            params![
                feed.id,
                feed.title,
                feed.url,
                feed.site_url,
                feed.folder,
                feed.custom_icon,
                feed.last_updated.map(|d| d.timestamp()),
                feed.error
            ],
        )?;
        Ok(())
    }

    pub fn delete_feed(&self, feed_id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM articles WHERE feed_id = ?1", params![feed_id])?;
        conn.execute("DELETE FROM feeds WHERE id = ?1", params![feed_id])?;
        Ok(())
    }

    pub fn delete_folder(&self, folder_name: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM articles WHERE feed_id IN (SELECT id FROM feeds WHERE folder = ?1)", params![folder_name])?;
        conn.execute("DELETE FROM feeds WHERE folder = ?1", params![folder_name])?;
        conn.execute("DELETE FROM folders WHERE name = ?1", params![folder_name])?;
        Ok(())
    }

    pub fn set_feed_folder(&self, feed_id: &str, folder: Option<&str>) -> Result<()> {
        let conn = self.conn();
        conn.execute("UPDATE feeds SET folder = ?1 WHERE id = ?2", params![folder, feed_id])?;
        Ok(())
    }

    /// Load the article list for a view.
    ///
    /// Reads no BLOB columns at all: the pane needs a title, a source, a time
    /// and a one-line preview, and all of those are plain columns. Bodies and
    /// summaries stay compressed in the table until the reader asks for one
    /// (see [`Database::get_article_body`]), so `Article::content` is always
    /// `None` here and `Article::summary` holds the short `snippet`.
    ///
    /// Every variant orders by the stored `sort_ts` so SQLite walks an index
    /// instead of sorting the result set.
    pub fn get_articles_by_filter(&self, filter: &CurrentFilter) -> Result<Vec<Article>> {
        let conn = self.conn();
        let now = Utc::now();
        let start_of_today = (now - chrono::Duration::hours(24)).timestamp();

        let query = match filter {
            CurrentFilter::Smart(SmartFeedKind::Today) => {
                "SELECT id, feed_id, feed_title, title, author, snippet, url, published, read, starred, created_at
                 FROM articles
                 WHERE sort_ts >= ?1
                 ORDER BY sort_ts DESC"
            }
            CurrentFilter::Smart(SmartFeedKind::AllUnread) => {
                "SELECT id, feed_id, feed_title, title, author, snippet, url, published, read, starred, created_at
                 FROM articles
                 WHERE read = 0
                 ORDER BY sort_ts DESC"
            }
            CurrentFilter::Smart(SmartFeedKind::Starred) => {
                "SELECT id, feed_id, feed_title, title, author, snippet, url, published, read, starred, created_at
                 FROM articles
                 WHERE starred = 1
                 ORDER BY sort_ts DESC"
            }
            CurrentFilter::Smart(SmartFeedKind::AllArticles) => {
                "SELECT id, feed_id, feed_title, title, author, snippet, url, published, read, starred, created_at
                 FROM articles
                 ORDER BY sort_ts DESC"
            }
            CurrentFilter::Folder(_) => {
                "SELECT a.id, a.feed_id, a.feed_title, a.title, a.author, a.snippet, a.url, a.published, a.read, a.starred, a.created_at
                 FROM articles a
                 JOIN feeds f ON a.feed_id = f.id
                 WHERE f.folder = ?1
                 ORDER BY a.sort_ts DESC"
            }
            CurrentFilter::Feed(_) => {
                "SELECT id, feed_id, feed_title, title, author, snippet, url, published, read, starred, created_at
                 FROM articles
                 WHERE feed_id = ?1
                 ORDER BY sort_ts DESC"
            }
        };
        let mut stmt = conn.prepare_cached(query)?;
        let mut rows = match filter {
            CurrentFilter::Smart(SmartFeedKind::Today) => stmt.query(params![start_of_today])?,
            CurrentFilter::Folder(name) => stmt.query(params![name])?,
            CurrentFilter::Feed(id) => stmt.query(params![id])?,
            _ => stmt.query([])?,
        };

        let mut articles = Vec::new();
        while let Some(row) = rows.next()? {
            let pub_ts: Option<i64> = row.get(7)?;
            let created_ts: i64 = row.get(10)?;
            let snippet: Option<String> = row.get(5)?;

            articles.push(Article {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                feed_title: row.get(2)?,
                title: row.get(3)?,
                author: row.get(4)?,
                summary: snippet.filter(|s| !s.is_empty()),
                content: None,
                url: row.get(6)?,
                published: pub_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                read: row.get::<_, i32>(8)? == 1,
                starred: row.get::<_, i32>(9)? == 1,
                created_at: DateTime::from_timestamp(created_ts, 0).unwrap_or(now),
            });
        }

        articles.shrink_to_fit();
        Ok(articles)
    }

    /// Fetch one article's body on demand, preferring `content` and falling
    /// back to `summary`. This is the only place a full body is decompressed.
    pub fn get_article_body(&self, article_id: &str) -> Option<String> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare_cached("SELECT content, summary FROM articles WHERE id = ?1")
            .ok()?;
        stmt.query_row(params![article_id], |row| {
            let content = read_body_column(row, 0)?;
            let summary = read_body_column(row, 1)?;
            Ok(content.filter(|c| !c.is_empty()).or(summary))
        })
        .optional()
        .ok()
        .flatten()
        .flatten()
    }

    pub fn insert_articles(&self, articles: &[Article]) -> Result<usize> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        let mut inserted_count = 0;

        // One prepared statement for the whole batch instead of re-parsing the
        // upsert per article; feed syncs push hundreds of rows at a time.
        let mut stmt = tx.prepare_cached(
            "INSERT INTO articles (id, feed_id, feed_title, title, author, summary, content, url, published, read, starred, created_at, sort_ts, snippet)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                 ON CONFLICT(id) DO UPDATE SET
                    feed_title = excluded.feed_title,
                    title = excluded.title,
                    author = COALESCE(excluded.author, articles.author),
                    summary = COALESCE(excluded.summary, articles.summary),
                    content = COALESCE(excluded.content, articles.content),
                    url = excluded.url,
                    published = COALESCE(excluded.published, articles.published),
                    sort_ts = excluded.sort_ts,
                    snippet = COALESCE(excluded.snippet, articles.snippet)",
        )?;

        for a in articles {
            let compressed_summary = a.summary.as_deref().map(compress_text);
            let compressed_content = a.content.as_deref().map(compress_text);

            let snippet = make_snippet(a.summary.as_deref(), a.content.as_deref());
            let sort_ts = a.published.unwrap_or(a.created_at).timestamp();

            let changed = stmt.execute(params![
                a.id,
                a.feed_id,
                a.feed_title,
                a.title,
                a.author,
                compressed_summary,
                compressed_content,
                a.url,
                a.published.map(|d| d.timestamp()),
                if a.read { 1 } else { 0 },
                if a.starred { 1 } else { 0 },
                a.created_at.timestamp(),
                sort_ts,
                snippet,
            ])?;
            inserted_count += changed;
        }

        drop(stmt);
        tx.commit()?;
        Ok(inserted_count)
    }

    pub fn set_article_read(&self, article_id: &str, read: bool) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE articles SET read = ?1 WHERE id = ?2",
            params![if read { 1 } else { 0 }, article_id],
        )?;
        Ok(())
    }

    pub fn set_article_starred(&self, article_id: &str, starred: bool) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE articles SET starred = ?1 WHERE id = ?2",
            params![if starred { 1 } else { 0 }, article_id],
        )?;
        Ok(())
    }

    pub fn mark_all_read_in_filter(&self, filter: &CurrentFilter) -> Result<()> {
        let conn = self.conn();
        let now = Utc::now();
        let start_of_today = (now - chrono::Duration::hours(24)).timestamp();

        match filter {
            CurrentFilter::Smart(SmartFeedKind::Today) => {
                conn.execute(
                    "UPDATE articles SET read = 1 WHERE sort_ts >= ?1",
                    params![start_of_today],
                )?;
            }
            CurrentFilter::Smart(SmartFeedKind::AllUnread) | CurrentFilter::Smart(SmartFeedKind::AllArticles) => {
                conn.execute("UPDATE articles SET read = 1", [])?;
            }
            CurrentFilter::Smart(SmartFeedKind::Starred) => {
                conn.execute("UPDATE articles SET read = 1 WHERE starred = 1", [])?;
            }
            CurrentFilter::Folder(folder) => {
                conn.execute(
                    "UPDATE articles SET read = 1 WHERE feed_id IN (SELECT id FROM feeds WHERE folder = ?1)",
                    params![folder],
                )?;
            }
            CurrentFilter::Feed(feed_id) => {
                conn.execute("UPDATE articles SET read = 1 WHERE feed_id = ?1", params![feed_id])?;
            }
        }
        Ok(())
    }

    pub fn is_folder_expanded(&self, folder_name: &str) -> bool {
        let conn = self.conn();
        let expanded: Option<i32> = conn
            .query_row(
                "SELECT is_expanded FROM folders WHERE name = ?1",
                params![folder_name],
                |r| r.get(0),
            )
            .optional()
            .unwrap_or(None);

        expanded.map(|v| v == 1).unwrap_or(true)
    }

    /// Every collapsed folder, in one query.
    ///
    /// Rebuilding the sidebar used to call [`Database::is_folder_expanded`]
    /// once per folder; folders default to expanded, so only the collapsed ones
    /// need to be known and they are almost always a short list.
    pub fn get_collapsed_folders(&self) -> HashSet<String> {
        let conn = self.conn();
        let mut collapsed = HashSet::new();
        if let Ok(mut stmt) =
            conn.prepare_cached("SELECT name FROM folders WHERE is_expanded = 0")
        {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                collapsed.extend(rows.flatten());
            }
        }
        collapsed
    }

    pub fn toggle_folder_expanded(&self, folder_name: &str) -> Result<bool> {
        // Read the current state before taking the lock: the guard is not
        // reentrant, so calling is_folder_expanded() while holding it deadlocks.
        let new_state = !self.is_folder_expanded(folder_name);
        let conn = self.conn();

        conn.execute(
            "INSERT INTO folders (name, is_expanded) VALUES (?1, ?2)
             ON CONFLICT(name) DO UPDATE SET is_expanded = excluded.is_expanded",
            params![folder_name, if new_state { 1 } else { 0 }],
        )?;
        
        Ok(new_state)
    }

    pub fn get_unread_counts(&self) -> Result<UnreadCounts> {
        let conn = self.conn();
        let now = Utc::now();
        let start_of_today = (now - chrono::Duration::hours(24)).timestamp();

        let mut counts = UnreadCounts::default();

        // 1. Smart feeds counts in a single efficient query
        let (today, all_unread, starred, all_articles): (i64, i64, i64, i64) = conn
            .query_row(
                "SELECT
                    COALESCE(SUM(CASE WHEN read = 0 AND sort_ts >= ?1 THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN read = 0 THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN starred = 1 THEN 1 ELSE 0 END), 0),
                    COUNT(*)
                 FROM articles",
                params![start_of_today],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap_or((0, 0, 0, 0));

        counts.today = today as usize;
        counts.all_unread = all_unread as usize;
        counts.starred = starred as usize;
        counts.all_articles = all_articles as usize;

        // 2. Feed counts: (unread, total)
        let mut stmt = conn.prepare(
            "SELECT feed_id, 
                    SUM(CASE WHEN read = 0 THEN 1 ELSE 0 END) as unread,
                    COUNT(*) as total
             FROM articles
             GROUP BY feed_id",
        )?;
        let feed_rows = stmt.query_map([], |r| {
            let feed_id: String = r.get(0)?;
            let unread: usize = r.get::<_, i64>(1)? as usize;
            let total: usize = r.get::<_, i64>(2)? as usize;
            Ok((feed_id, (unread, total)))
        })?;

        for row in feed_rows {
            if let Ok((feed_id, tuple)) = row {
                counts.feed_counts.insert(feed_id, tuple);
            }
        }

        // Folder counts: (unread, total)
        let mut folder_stmt = conn.prepare(
            "SELECT f.folder,
                    SUM(CASE WHEN a.read = 0 THEN 1 ELSE 0 END) as unread,
                    COUNT(a.id) as total
             FROM feeds f
             LEFT JOIN articles a ON f.id = a.feed_id
             WHERE f.folder IS NOT NULL AND f.folder != ''
             GROUP BY f.folder"
        )?;
        let folder_rows = folder_stmt.query_map([], |r| {
            let folder: String = r.get(0)?;
            let unread: usize = r.get::<_, i64>(1).unwrap_or(0) as usize;
            let total: usize = r.get::<_, i64>(2).unwrap_or(0) as usize;
            Ok((folder, (unread, total)))
        })?;

        for row in folder_rows {
            if let Ok((folder, tuple)) = row {
                counts.folder_counts.insert(folder, tuple);
            }
        }

        Ok(counts)
    }
}
