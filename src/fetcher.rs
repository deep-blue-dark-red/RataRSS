use crate::model::{Article, Feed};
use crate::opml::generate_feed_id;
use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct FeedFetcher {
    client: Client,
}

impl Default for FeedFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(8))
            .redirect(reqwest::redirect::Policy::limited(5))
            .user_agent("ratarss/0.1.0 (https://github.com/ratarss/ratarss; RSS reader)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    pub async fn fetch_feed(&self, feed: &Feed) -> Result<(Feed, Vec<Article>), String> {
        let response = self
            .client
            .get(&feed.url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP error {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("")));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        // Attempt feed-rs parsing
        let parsed_feed = match feed_rs::parser::parse(&bytes[..]) {
            Ok(f) => f,
            Err(e) => {
                // If it failed to parse as feed, check if it's HTML with auto-discovery
                if let Ok(html_str) = String::from_utf8(bytes.to_vec()) {
                    if let Some(discovered_url) = discover_feed_url(&html_str, &feed.url) {
                        let discovered_resp = self
                            .client
                            .get(&discovered_url)
                            .send()
                            .await
                            .map_err(|e| format!("Discovered feed request failed: {}", e))?;
                        let disc_bytes = discovered_resp.bytes().await.map_err(|e| e.to_string())?;
                        feed_rs::parser::parse(&disc_bytes[..])
                            .map_err(|e| format!("Failed to parse discovered feed: {}", e))?
                    } else {
                        return Err(format!("Feed parse error: {}", e));
                    }
                } else {
                    return Err(format!("Feed parse error: {}", e));
                }
            }
        };

        let mut updated_feed = feed.clone();
        if feed.title.is_empty() || feed.title == "Untitled Feed" {
            if let Some(ref title) = parsed_feed.title {
                updated_feed.title = clean_html_tags(&title.content);
            }
        }

        if updated_feed.site_url.is_none() {
            if let Some(link) = parsed_feed.links.first() {
                updated_feed.site_url = Some(link.href.clone());
            }
        }

        updated_feed.last_updated = Some(Utc::now());
        updated_feed.error = None;

        let mut articles = Vec::new();
        let now = Utc::now();

        for entry in parsed_feed.entries {
            let entry_id = if !entry.id.is_empty() {
                format!("{}-{}", feed.id, entry.id)
            } else if let Some(first_link) = entry.links.first() {
                format!("{}-{}", feed.id, first_link.href)
            } else {
                format!("{}-{}", feed.id, uuid::Uuid::new_v4())
            };

            let title = entry
                .title
                .map(|t| clean_html_tags(&t.content))
                .unwrap_or_else(|| "Untitled Article".to_string());

            let author = entry.authors.first().map(|a| a.name.clone());

            let mut summary = entry.summary.map(|s| clean_html_tags(&s.content));
            
            let mut content_html = None;
            if let Some(content) = entry.content {
                if let Some(body) = content.body {
                    content_html = Some(body.clone());
                    if summary.is_none() {
                        summary = Some(generate_snippet(&body, 280));
                    }
                }
            }

            if summary.is_none() && content_html.is_none() {
                summary = Some(String::new());
            }

            let link_url = entry
                .links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_else(|| feed.site_url.clone().unwrap_or_else(|| feed.url.clone()));

            let published: Option<DateTime<Utc>> = entry.published.or(entry.updated);

            articles.push(Article {
                id: entry_id,
                feed_id: feed.id.clone(),
                feed_title: updated_feed.title.clone(),
                title,
                author,
                summary,
                content: content_html,
                url: link_url,
                published,
                read: false,
                starred: false,
                created_at: now,
            });
        }

        Ok((updated_feed, articles))
    }

    pub async fn discover_or_create_feed(&self, url: &str, folder: Option<String>) -> Result<(Feed, Vec<Article>), String> {
        let temp_feed = Feed::new(
            generate_feed_id(url),
            "Loading feed...".to_string(),
            url.to_string(),
            None,
            folder,
        );

        self.fetch_feed(&temp_feed).await
    }
}

pub fn clean_html_tags(input: &str) -> String {
    let unescaped = html_escape::decode_html_entities(input);
    let re_tags = Regex::new(r"<[^>]*>").unwrap();
    let stripped = re_tags.replace_all(&unescaped, " ");
    let re_spaces = Regex::new(r"\s+").unwrap();
    re_spaces.replace_all(&stripped, " ").trim().to_string()
}

pub fn generate_snippet(input: &str, max_len: usize) -> String {
    let clean = clean_html_tags(input);
    if clean.chars().count() <= max_len {
        clean
    } else {
        let truncated: String = clean.chars().take(max_len).collect();
        format!("{}…", truncated.trim_end())
    }
}

pub fn discover_feed_url(html: &str, base_url: &str) -> Option<String> {
    let re_link = Regex::new(r#"(?i)<link\s+[^>]*type=["']application/(rss\+xml|atom\+xml|json)["'][^>]*>"#).ok()?;
    let re_href = Regex::new(r#"(?i)href=["']([^"']+)["']"#).ok()?;

    if let Some(mat) = re_link.find(html) {
        let tag = mat.as_str();
        if let Some(caps) = re_href.captures(tag) {
            let href = caps.get(1)?.as_str();
            return Some(resolve_url(base_url, href));
        }
    }

    // Try common feed paths
    if let Ok(parsed_base) = reqwest::Url::parse(base_url) {
        if let Ok(feed_url) = parsed_base.join("/feed") {
            return Some(feed_url.to_string());
        }
    }

    None
}

fn resolve_url(base: &str, relative: &str) -> String {
    if relative.starts_with("http://") || relative.starts_with("https://") {
        return relative.to_string();
    }
    if let Ok(base_parsed) = reqwest::Url::parse(base) {
        if let Ok(joined) = base_parsed.join(relative) {
            return joined.to_string();
        }
    }
    relative.to_string()
}
