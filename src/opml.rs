use crate::model::Feed;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use quick_xml::Writer;
use std::fs;
use std::io::Cursor;
use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OpmlOutline {

    pub text: String,
    pub title: Option<String>,
    pub xml_url: Option<String>,
    pub html_url: Option<String>,
    pub folder: Option<String>,
}

pub fn parse_opml_str(xml_content: &str) -> Result<Vec<Feed>, String> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut feeds = Vec::new();
    let mut folder_stack: Vec<String> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"outline" {
                    let (display_title, xml_url, html_url) = parse_outline_attrs(e);
                    if !xml_url.is_empty() {
                        let folder = folder_stack.last().cloned();
                        let id = generate_feed_id(&xml_url);
                        feeds.push(Feed::new(
                            id,
                            display_title,
                            xml_url,
                            if html_url.is_empty() { None } else { Some(html_url) },
                            folder,
                        ));
                    } else if !display_title.is_empty() {
                        folder_stack.push(display_title);
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"outline" {
                    let (display_title, xml_url, html_url) = parse_outline_attrs(e);
                    if !xml_url.is_empty() {
                        let folder = folder_stack.last().cloned();
                        let id = generate_feed_id(&xml_url);
                        feeds.push(Feed::new(
                            id,
                            display_title,
                            xml_url,
                            if html_url.is_empty() { None } else { Some(html_url) },
                            folder,
                        ));
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"outline" {
                    folder_stack.pop();
                }
            }
            Ok(Event::Eof) => break,
            Err(err) => return Err(format!("OPML XML parse error: {}", err)),
            _ => {}
        }
        buf.clear();
    }

    Ok(feeds)
}

fn parse_outline_attrs(e: &BytesStart) -> (String, String, String) {
    let mut text = String::new();
    let mut title = String::new();
    let mut xml_url = String::new();
    let mut html_url = String::new();

    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
        let val = attr.unescape_value().unwrap_or_default().to_string();

        match key.as_str() {
            "text" => text = val,
            "title" => title = val,
            "xmlurl" | "xml_url" => xml_url = val,
            "htmlurl" | "html_url" => html_url = val,
            _ => {}
        }
    }

    let display_title = if !title.is_empty() {
        title
    } else if !text.is_empty() {
        text
    } else {
        "Untitled Feed".to_string()
    };

    (display_title, xml_url, html_url)
}

pub fn parse_opml_file<P: AsRef<Path>>(path: P) -> Result<Vec<Feed>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read OPML file: {}", e))?;
    parse_opml_str(&content)
}

pub fn export_opml(feeds: &[Feed], title: &str) -> Result<String, String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // XML Declaration
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| e.to_string())?;

    // <opml version="2.0">
    let mut opml_start = BytesStart::new("opml");
    opml_start.push_attribute(("version", "2.0"));
    writer.write_event(Event::Start(opml_start)).map_err(|e| e.to_string())?;

    // <head><title>...</title></head>
    writer.write_event(Event::Start(BytesStart::new("head"))).map_err(|e| e.to_string())?;
    writer.write_event(Event::Start(BytesStart::new("title"))).map_err(|e| e.to_string())?;
    writer
        .write_event(Event::Text(BytesText::new(title)))
        .map_err(|e| e.to_string())?;
    writer.write_event(Event::End(BytesEnd::new("title"))).map_err(|e| e.to_string())?;
    writer.write_event(Event::End(BytesEnd::new("head"))).map_err(|e| e.to_string())?;

    // <body>
    writer.write_event(Event::Start(BytesStart::new("body"))).map_err(|e| e.to_string())?;

    // Group feeds by folder
    let mut folder_map: std::collections::BTreeMap<Option<String>, Vec<&Feed>> = std::collections::BTreeMap::new();
    for f in feeds {
        folder_map.entry(f.folder.clone()).or_default().push(f);
    }

    for (folder, folder_feeds) in folder_map {
        match folder {
            Some(folder_name) => {
                let mut folder_start = BytesStart::new("outline");
                folder_start.push_attribute(("text", folder_name.as_str()));
                folder_start.push_attribute(("title", folder_name.as_str()));
                writer.write_event(Event::Start(folder_start)).map_err(|e| e.to_string())?;

                for feed in folder_feeds {
                    let mut feed_empty = BytesStart::new("outline");
                    feed_empty.push_attribute(("type", "rss"));
                    feed_empty.push_attribute(("text", feed.title.as_str()));
                    feed_empty.push_attribute(("title", feed.title.as_str()));
                    feed_empty.push_attribute(("xmlUrl", feed.url.as_str()));
                    if let Some(ref site) = feed.site_url {
                        feed_empty.push_attribute(("htmlUrl", site.as_str()));
                    }
                    writer.write_event(Event::Empty(feed_empty)).map_err(|e| e.to_string())?;
                }

                writer.write_event(Event::End(BytesEnd::new("outline"))).map_err(|e| e.to_string())?;
            }
            None => {
                for feed in folder_feeds {
                    let mut feed_empty = BytesStart::new("outline");
                    feed_empty.push_attribute(("type", "rss"));
                    feed_empty.push_attribute(("text", feed.title.as_str()));
                    feed_empty.push_attribute(("title", feed.title.as_str()));
                    feed_empty.push_attribute(("xmlUrl", feed.url.as_str()));
                    if let Some(ref site) = feed.site_url {
                        feed_empty.push_attribute(("htmlUrl", site.as_str()));
                    }
                    writer.write_event(Event::Empty(feed_empty)).map_err(|e| e.to_string())?;
                }
            }
        }
    }

    // </body > </opml>
    writer.write_event(Event::End(BytesEnd::new("body"))).map_err(|e| e.to_string())?;
    writer.write_event(Event::End(BytesEnd::new("opml"))).map_err(|e| e.to_string())?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| e.to_string())
}

pub fn generate_feed_id(url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("feed-{:016x}", hasher.finish())
}
