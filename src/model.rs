use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Feed {
    pub id: String,
    pub title: String,
    pub url: String,
    pub site_url: Option<String>,
    pub folder: Option<String>,
    pub custom_icon: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub unread_count: usize,
    pub total_count: usize,
}

impl Feed {
    pub fn new(id: String, title: String, url: String, site_url: Option<String>, folder: Option<String>) -> Self {
        Self {
            id,
            title,
            url,
            site_url,
            folder,
            custom_icon: None,
            last_updated: None,
            error: None,
            unread_count: 0,
            total_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Article {
    pub id: String,
    pub feed_id: String,
    pub feed_title: String,
    pub title: String,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub url: String,
    pub published: Option<DateTime<Utc>>,
    pub read: bool,
    pub starred: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SmartFeedKind {
    Today,
    AllUnread,
    Starred,
    AllArticles,
}

impl SmartFeedKind {
    pub fn title(&self) -> &'static str {
        match self {
            SmartFeedKind::Today => "Today",
            SmartFeedKind::AllUnread => "All Unread",
            SmartFeedKind::Starred => "Starred",
            SmartFeedKind::AllArticles => "All Articles",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SmartFeedKind::Today => "☀️",
            SmartFeedKind::AllUnread => "🔵",
            SmartFeedKind::Starred => "⭐",
            SmartFeedKind::AllArticles => "📜",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarItem {
    SmartHeader,
    Smart(SmartFeedKind, usize), // Kind, unread count
    FolderHeader {
        name: String,
        is_expanded: bool,
        unread_count: usize,
        feed_count: usize,
    },
    Feed {
        feed_id: String,
        title: String,
        folder: Option<String>,
        unread_count: usize,
        has_error: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivePane {
    Sidebar,
    ArticleList,
    Reader,
}

impl ActivePane {
    pub fn next(&self) -> Self {
        match self {
            ActivePane::Sidebar => ActivePane::ArticleList,
            ActivePane::ArticleList => ActivePane::Reader,
            ActivePane::Reader => ActivePane::Sidebar,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            ActivePane::Sidebar => ActivePane::Reader,
            ActivePane::ArticleList => ActivePane::Sidebar,
            ActivePane::Reader => ActivePane::ArticleList,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrentFilter {
    Smart(SmartFeedKind),
    Folder(String),
    Feed(String), // feed_id
}
