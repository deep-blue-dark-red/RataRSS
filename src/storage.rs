use crate::config::AppConfig;
use crate::model::{Article, CurrentFilter, Feed, SmartFeedKind};
use crate::sample_data;
use chrono::{DateTime, Utc};
use rusqlite::types::ValueRef;
use rusqlite::{params, Connection, OptionalExtension, Result, Row};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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

#[derive(Clone)]
pub struct Database {
    db_path: PathBuf,
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
        let db = Self { db_path };
        db.init_schema()?;
        let _ = db.compress_uncompressed_articles();
        Ok(db)
    }

    pub fn in_memory() -> Result<Self> {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("ratarss_test_{}.db", uuid::Uuid::new_v4()));
        let db = Self { db_path };
        db.init_schema()?;
        Ok(db)
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.db_path
    }

    fn get_conn(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 10000;
             PRAGMA foreign_keys = ON;
             PRAGMA cache_size = -32000;
             PRAGMA temp_store = MEMORY;"
        )?;
        Ok(conn)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.get_conn()?;
        
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
            CREATE INDEX IF NOT EXISTS idx_articles_published ON articles(published DESC);
            CREATE INDEX IF NOT EXISTS idx_articles_read ON articles(read);
            CREATE INDEX IF NOT EXISTS idx_articles_starred ON articles(starred);
            CREATE INDEX IF NOT EXISTS idx_articles_created ON articles(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_feeds_folder ON feeds(folder);
            "
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

                conn.execute(
                    "INSERT OR IGNORE INTO articles (id, feed_id, feed_title, title, author, summary, content, url, published, read, starred, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
                    ],
                )?;
            }
        }

        Ok(())
    }

    /// Automatically migrates and compresses any legacy uncompressed articles using Zstandard
    pub fn compress_uncompressed_articles(&self) -> Result<usize> {
        let mut conn = self.get_conn()?;
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
        Ok(updated)
    }

    pub fn get_db_stats(&self) -> Result<DatabaseStats> {
        let conn = self.get_conn()?;
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
        let conn = self.get_conn()?;
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
        let conn = self.get_conn()?;
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
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM articles WHERE feed_id = ?1", params![feed_id])?;
        conn.execute("DELETE FROM feeds WHERE id = ?1", params![feed_id])?;
        Ok(())
    }

    pub fn delete_folder(&self, folder_name: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("DELETE FROM articles WHERE feed_id IN (SELECT id FROM feeds WHERE folder = ?1)", params![folder_name])?;
        conn.execute("DELETE FROM feeds WHERE folder = ?1", params![folder_name])?;
        conn.execute("DELETE FROM folders WHERE name = ?1", params![folder_name])?;
        Ok(())
    }

    pub fn set_feed_folder(&self, feed_id: &str, folder: Option<&str>) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("UPDATE feeds SET folder = ?1 WHERE id = ?2", params![folder, feed_id])?;
        Ok(())
    }

    pub fn get_articles_by_filter(&self, filter: &CurrentFilter) -> Result<Vec<Article>> {
        let conn = self.get_conn()?;
        let now = Utc::now();
        let start_of_today = (now - chrono::Duration::hours(24)).timestamp();

        let query = match filter {
            CurrentFilter::Smart(SmartFeedKind::Today) => {
                "SELECT id, feed_id, feed_title, title, author, summary, content, url, published, read, starred, created_at
                 FROM articles
                 WHERE published >= ?1 OR created_at >= ?1
                 ORDER BY COALESCE(published, created_at) DESC"
            }
            CurrentFilter::Smart(SmartFeedKind::AllUnread) => {
                "SELECT id, feed_id, feed_title, title, author, summary, content, url, published, read, starred, created_at
                 FROM articles
                 WHERE read = 0
                 ORDER BY COALESCE(published, created_at) DESC"
            }
            CurrentFilter::Smart(SmartFeedKind::Starred) => {
                "SELECT id, feed_id, feed_title, title, author, summary, content, url, published, read, starred, created_at
                 FROM articles
                 WHERE starred = 1
                 ORDER BY COALESCE(published, created_at) DESC"
            }
            CurrentFilter::Smart(SmartFeedKind::AllArticles) => {
                "SELECT id, feed_id, feed_title, title, author, summary, content, url, published, read, starred, created_at
                 FROM articles
                 ORDER BY COALESCE(published, created_at) DESC"
            }
            CurrentFilter::Folder(_) => {
                "SELECT a.id, a.feed_id, a.feed_title, a.title, a.author, a.summary, a.content, a.url, a.published, a.read, a.starred, a.created_at
                 FROM articles a
                 JOIN feeds f ON a.feed_id = f.id
                 WHERE f.folder = ?1
                 ORDER BY COALESCE(a.published, a.created_at) DESC"
            }
            CurrentFilter::Feed(_) => {
                "SELECT id, feed_id, feed_title, title, author, summary, content, url, published, read, starred, created_at
                 FROM articles
                 WHERE feed_id = ?1
                 ORDER BY COALESCE(published, created_at) DESC"
            }
        };

        let mut stmt = conn.prepare(query)?;
        let mut rows = match filter {
            CurrentFilter::Smart(SmartFeedKind::Today) => stmt.query(params![start_of_today])?,
            CurrentFilter::Folder(name) => stmt.query(params![name])?,
            CurrentFilter::Feed(id) => stmt.query(params![id])?,
            _ => stmt.query([])?,
        };

        let mut articles = Vec::new();
        while let Some(row) = rows.next()? {
            let pub_ts: Option<i64> = row.get(8)?;
            let created_ts: i64 = row.get(11)?;
            let summary = read_body_column(row, 5)?;
            let content = read_body_column(row, 6)?;

            articles.push(Article {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                feed_title: row.get(2)?,
                title: row.get(3)?,
                author: row.get(4)?,
                summary,
                content,
                url: row.get(7)?,
                published: pub_ts.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                read: row.get::<_, i32>(9)? == 1,
                starred: row.get::<_, i32>(10)? == 1,
                created_at: DateTime::from_timestamp(created_ts, 0).unwrap_or(now),
            });
        }

        Ok(articles)
    }

    pub fn insert_articles(&self, articles: &[Article]) -> Result<usize> {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction()?;
        let mut inserted_count = 0;

        for a in articles {
            let compressed_summary = a.summary.as_deref().map(compress_text);
            let compressed_content = a.content.as_deref().map(compress_text);

            let changed = tx.execute(
                "INSERT INTO articles (id, feed_id, feed_title, title, author, summary, content, url, published, read, starred, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(id) DO UPDATE SET
                    feed_title = excluded.feed_title,
                    title = excluded.title,
                    author = COALESCE(excluded.author, articles.author),
                    summary = COALESCE(excluded.summary, articles.summary),
                    content = COALESCE(excluded.content, articles.content),
                    url = excluded.url,
                    published = COALESCE(excluded.published, articles.published)",
                params![
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
                ],
            )?;
            inserted_count += changed;
        }

        tx.commit()?;
        Ok(inserted_count)
    }

    pub fn set_article_read(&self, article_id: &str, read: bool) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE articles SET read = ?1 WHERE id = ?2",
            params![if read { 1 } else { 0 }, article_id],
        )?;
        Ok(())
    }

    pub fn set_article_starred(&self, article_id: &str, starred: bool) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "UPDATE articles SET starred = ?1 WHERE id = ?2",
            params![if starred { 1 } else { 0 }, article_id],
        )?;
        Ok(())
    }

    pub fn mark_all_read_in_filter(&self, filter: &CurrentFilter) -> Result<()> {
        let conn = self.get_conn()?;
        let now = Utc::now();
        let start_of_today = (now - chrono::Duration::hours(24)).timestamp();

        match filter {
            CurrentFilter::Smart(SmartFeedKind::Today) => {
                conn.execute(
                    "UPDATE articles SET read = 1 WHERE published >= ?1 OR created_at >= ?1",
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
        let conn = match self.get_conn() {
            Ok(c) => c,
            Err(_) => return true,
        };
        
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

    pub fn toggle_folder_expanded(&self, folder_name: &str) -> Result<bool> {
        let conn = self.get_conn()?;
        let current = self.is_folder_expanded(folder_name);
        let new_state = !current;
        
        conn.execute(
            "INSERT INTO folders (name, is_expanded) VALUES (?1, ?2)
             ON CONFLICT(name) DO UPDATE SET is_expanded = excluded.is_expanded",
            params![folder_name, if new_state { 1 } else { 0 }],
        )?;
        
        Ok(new_state)
    }

    pub fn get_unread_counts(&self) -> Result<UnreadCounts> {
        let conn = self.get_conn()?;
        let now = Utc::now();
        let start_of_today = (now - chrono::Duration::hours(24)).timestamp();

        let mut counts = UnreadCounts::default();

        // 1. Smart feeds counts in a single efficient query
        let (today, all_unread, starred, all_articles): (i64, i64, i64, i64) = conn
            .query_row(
                "SELECT
                    COALESCE(SUM(CASE WHEN read = 0 AND (published >= ?1 OR created_at >= ?1) THEN 1 ELSE 0 END), 0),
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
