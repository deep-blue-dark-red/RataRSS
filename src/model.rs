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

/// A borrowed view of the article list as it is currently displayed: the full
/// list, plus an optional index of the rows a search matched.
///
/// The list pane used to receive a freshly cloned `Vec<Article>` every frame,
/// which copied every title, summary and body in the view. This carries two
/// pointers instead.
#[derive(Clone, Copy)]
pub struct ArticleSlice<'a> {
    pub all: &'a [Article],
    pub filtered: Option<&'a [u32]>,
}

impl<'a> ArticleSlice<'a> {
    pub fn len(&self) -> usize {
        match self.filtered {
            Some(idx) => idx.len(),
            None => self.all.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, pos: usize) -> Option<&'a Article> {
        match self.filtered {
            Some(idx) => idx.get(pos).and_then(|i| self.all.get(*i as usize)),
            None => self.all.get(pos),
        }
    }
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
