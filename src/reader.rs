use crate::model::Article;
use crate::theme::Theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use regex::Regex;
use std::sync::OnceLock;

/// Compiling a regex costs far more than running it, and these two used to be
/// rebuilt for every paragraph of every article on every frame.
fn tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<[^>]*>").unwrap())
}

fn space_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\s+").unwrap())
}

fn anchor_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?i)<a\s+[^>]*href=["']([^"']+)["'][^>]*>(.*?)</a>"#).unwrap())
}

/// A clickable region: where a link's text was laid out after wrapping.
///
/// Columns are relative to the start of the text area, so the reader can map a
/// click straight onto it without knowing anything about how the line wrapped.
#[derive(Debug, Clone, Copy)]
pub struct LinkHit {
    pub line: usize,
    pub start_col: u16,
    pub end_col: u16,
    /// Index into [`FormattedArticle::links`].
    pub link: usize,
}

#[allow(dead_code)]
pub struct FormattedArticle {

    pub lines: Vec<Line<'static>>,
    pub links: Vec<String>,
    pub total_lines: usize,
    /// Every laid-out link region, in line order.
    pub hits: Vec<LinkHit>,
}

impl FormattedArticle {
    /// The URL at a text-area coordinate, if a link was drawn there.
    pub fn link_at(&self, line: usize, col: u16) -> Option<&str> {
        self.hits
            .iter()
            .find(|h| h.line == line && col >= h.start_col && col < h.end_col)
            .and_then(|h| self.links.get(h.link))
            .map(String::as_str)
    }
}

/// Format an article for the reader pane.
///
/// `body` is the article text loaded on demand (see
/// `Database::get_article_body`); the list keeps `Article::content` empty so
/// bodies never sit in memory for every row.
pub fn render_article_to_text(
    article: &Article,
    body: Option<&str>,
    theme: &Theme,
    max_width: u16,
) -> FormattedArticle {
    let mut lines = Vec::new();
    let mut links = Vec::new();
    let mut hits: Vec<LinkHit> = Vec::new();
    let width = if max_width > 4 { (max_width - 4) as usize } else { 80 };

    // --- Header Section ---
    // Feed Name and Author
    let feed_author_line = match &article.author {
        Some(author) if !author.is_empty() => Line::from(vec![
            Span::styled(
                article.feed_title.to_uppercase(),
                Style::default().fg(theme.reader_header_feed).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" • ", Style::default().fg(theme.fg_subtle)),
            Span::styled(
                author.clone(),
                Style::default().fg(theme.reader_header_author).add_modifier(Modifier::ITALIC),
            ),
        ]),
        _ => Line::from(vec![Span::styled(
            article.feed_title.to_uppercase(),
            Style::default().fg(theme.reader_header_feed).add_modifier(Modifier::BOLD),
        )]),
    };
    lines.push(feed_author_line);
    lines.push(Line::from(""));

    // Title (Large & Bold, wrapped)
    let title_lines = wrap_text(&article.title, width);
    for tl in title_lines {
        lines.push(Line::from(vec![Span::styled(
            tl,
            Style::default()
                .fg(theme.reader_title)
                .add_modifier(Modifier::BOLD),
        )]));
    }
    lines.push(Line::from(""));

    // Date & Time + URL
    let date_str = if let Some(pub_date) = article.published {
        pub_date.format("%d %b %Y at %I:%M %p UTC").to_string().to_uppercase()
    } else {
        "DATE UNKNOWN".to_string()
    };

    let meta_line = Line::from(vec![
        Span::styled(date_str, Style::default().fg(theme.reader_meta)),
    ]);
    lines.push(meta_line);

    // Link URL line — clickable, like the links in the body.
    if !article.url.is_empty() {
        links.push(article.url.clone());
        let label = truncate_string(&article.url, width.saturating_sub(4));
        let label_width = unicode_width::UnicodeWidthStr::width(label.as_str()) as u16;
        let icon = "🔗 ";
        let icon_width = unicode_width::UnicodeWidthStr::width(icon) as u16;
        hits.push(LinkHit {
            line: lines.len(),
            start_col: icon_width,
            end_col: icon_width + label_width,
            link: links.len() - 1,
        });
        lines.push(Line::from(vec![
            Span::styled(icon, Style::default().fg(theme.reader_link)),
            Span::styled(
                label,
                Style::default().fg(theme.reader_link_url).add_modifier(Modifier::UNDERLINED),
            ),
        ]));
    }

    // Subtle divider
    lines.push(Line::from(""));
    let divider_str = "─".repeat(width.min(80));
    lines.push(Line::from(vec![Span::styled(
        divider_str,
        Style::default().fg(theme.reader_divider),
    )]));
    lines.push(Line::from(""));

    // --- Content Section ---
    let raw_content = body
        .or(article.content.as_deref())
        .or(article.summary.as_deref())
        .unwrap_or("No content available for this article.");

    let body_lines =
        parse_html_with_links(raw_content, theme, width, lines.len(), &mut links, &mut hits);
    lines.extend(body_lines);

    // Add footer margin
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    let total_lines = lines.len();
    FormattedArticle {
        lines,
        links,
        total_lines,
        hits,
    }
}

#[allow(dead_code)]
pub fn parse_html_to_ratatui_lines(
    html: &str,
    theme: &Theme,
    width: usize,
    links: &mut Vec<String>,
) -> Vec<Line<'static>> {
    let mut hits = Vec::new();
    parse_html_with_links(html, theme, width, 0, links, &mut hits)
}

/// As [`parse_html_to_ratatui_lines`], but also reporting where each link was
/// laid out. `base_line` is the index the first produced line will occupy in
/// the finished document, so recorded hits are absolute.
pub fn parse_html_with_links(
    html: &str,
    theme: &Theme,
    width: usize,
    base_line: usize,
    links: &mut Vec<String>,
    hits: &mut Vec<LinkHit>,
) -> Vec<Line<'static>> {
    let mut result = Vec::new();
    let unescaped = html_escape::decode_html_entities(html);
    let normalized = normalize_block_tags(&unescaped);

    let paragraphs = normalized.split("\n\n");

    for p in paragraphs {
        let p_trimmed = p.trim();
        if p_trimmed.is_empty() {
            continue;
        }

        // Check if heading
        if p_trimmed.starts_with("<h1") || p_trimmed.starts_with("# ") {
            let clean = strip_tags(p_trimmed);
            let clean_h = clean.trim_start_matches('#').trim();
            result.push(Line::from(""));
            for w in wrap_text(clean_h, width) {
                result.push(Line::from(vec![Span::styled(
                    w,
                    Style::default().fg(theme.reader_h1).add_modifier(Modifier::BOLD),
                )]));
            }
            result.push(Line::from(""));
            continue;
        }

        if p_trimmed.starts_with("<h2") || p_trimmed.starts_with("## ") {
            let clean = strip_tags(p_trimmed);
            let clean_h = clean.trim_start_matches('#').trim();
            result.push(Line::from(""));
            for w in wrap_text(clean_h, width) {
                result.push(Line::from(vec![Span::styled(
                    w,
                    Style::default().fg(theme.reader_h2).add_modifier(Modifier::BOLD),
                )]));
            }
            result.push(Line::from(""));
            continue;
        }

        if p_trimmed.starts_with("<h3") || p_trimmed.starts_with("### ") {
            let clean = strip_tags(p_trimmed);
            let clean_h = clean.trim_start_matches('#').trim();
            result.push(Line::from(""));
            for w in wrap_text(clean_h, width) {
                result.push(Line::from(vec![Span::styled(
                    w,
                    Style::default().fg(theme.reader_h3).add_modifier(Modifier::BOLD),
                )]));
            }
            result.push(Line::from(""));
            continue;
        }

        // Check if blockquote
        if p_trimmed.starts_with("<blockquote") || p_trimmed.starts_with("> ") {
            let clean = strip_tags(p_trimmed);
            let clean_q = clean.trim_start_matches('>').trim();
            for w in wrap_text(clean_q, width.saturating_sub(4)) {
                result.push(Line::from(vec![
                    Span::styled("│ ", Style::default().fg(theme.reader_quote_border).add_modifier(Modifier::BOLD)),
                    Span::styled(w, Style::default().fg(theme.reader_quote_fg).add_modifier(Modifier::ITALIC)),
                ]));
            }
            result.push(Line::from(""));
            continue;
        }

        // Check if code block
        if p_trimmed.starts_with("<pre") || p_trimmed.starts_with("```") {
            let clean = strip_tags(p_trimmed);
            let clean_code = clean.trim_matches('`').trim();
            result.push(Line::from(vec![
                Span::styled("┌── Code Snippet ", Style::default().fg(theme.reader_quote_border)),
                Span::styled("─".repeat(width.saturating_sub(18)), Style::default().fg(theme.reader_divider)),
            ]));
            for line in clean_code.lines() {
                result.push(Line::from(vec![
                    Span::styled("│ ", Style::default().fg(theme.reader_quote_border)),
                    Span::styled(line.to_string(), Style::default().fg(theme.reader_code_fg)),
                ]));
            }
            result.push(Line::from(vec![
                Span::styled("└──", Style::default().fg(theme.reader_quote_border)),
                Span::styled("─".repeat(width.saturating_sub(5)), Style::default().fg(theme.reader_divider)),
            ]));
            result.push(Line::from(""));
            continue;
        }

        // Check list items
        if p_trimmed.contains("<li") || p_trimmed.contains("\n* ") || p_trimmed.contains("\n- ") {
            for item in p_trimmed.split('\n') {
                let item_trimmed = item.trim();
                if item_trimmed.is_empty() {
                    continue;
                }
                let clean = strip_tags(item_trimmed);
                let text_body = clean
                    .trim_start_matches('*')
                    .trim_start_matches('-')
                    .trim_start_matches('•')
                    .trim();

                let wrapped = wrap_text(text_body, width.saturating_sub(4));
                for (idx, w) in wrapped.into_iter().enumerate() {
                    if idx == 0 {
                        result.push(Line::from(vec![
                            Span::styled("  • ", Style::default().fg(theme.accent)),
                            Span::styled(w, Style::default().fg(theme.reader_body)),
                        ]));
                    } else {
                        result.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled(w, Style::default().fg(theme.reader_body)),
                        ]));
                    }
                }
            }
            result.push(Line::from(""));
            continue;
        }

        // Standard Paragraph. Anchors stay as their own runs so they can be
        // styled as links and hit-tested after wrapping, instead of being
        // flattened into the surrounding prose.
        let segments = split_anchors(p_trimmed, links);
        for (line, line_hits) in wrap_segments(&segments, width, theme) {
            for hit in line_hits {
                hits.push(LinkHit {
                    line: base_line + result.len(),
                    start_col: hit.0,
                    end_col: hit.1,
                    link: hit.2,
                });
            }
            result.push(line);
        }
        result.push(Line::from(""));
    }

    if result.is_empty() {
        result.push(Line::from(vec![Span::styled(
            "No content preview available.",
            Style::default().fg(theme.fg_dim),
        )]));
    }

    result
}

/// Turn the block-level tags that imply a line break into newlines, in a single
/// pass.
///
/// The previous version chained fourteen `String::replace` calls, each of which
/// walked and reallocated the whole document.
fn normalize_block_tags(input: &str) -> String {
    // (tag, replacement) — matched case-insensitively against the `<` position.
    const BLOCKS: &[(&str, &str)] = &[
        ("<br>", "\n"),
        ("<br/>", "\n"),
        ("<br />", "\n"),
        ("</p>", "\n\n"),
        ("</div>", "\n"),
        ("</h1>", "\n\n"),
        ("</h2>", "\n\n"),
        ("</h3>", "\n\n"),
        ("</h4>", "\n\n"),
        ("</blockquote>", "\n\n"),
        ("</li>", "\n"),
        ("</ul>", "\n\n"),
        ("</ol>", "\n\n"),
        ("</pre>", "\n\n"),
    ];

    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let rest = &bytes[i..];
            // Compare as bytes: slicing the &str by a tag's byte length can cut
            // through a multi-byte character that follows the `<` and panic.
            // Every tag is ASCII, so a byte-wise match is exact.
            if let Some((tag, replacement)) = BLOCKS.iter().find(|(tag, _)| {
                rest.len() >= tag.len() && rest[..tag.len()].eq_ignore_ascii_case(tag.as_bytes())
            }) {
                out.push_str(replacement);
                i += tag.len();
                continue;
            }
        }
        // Copy one UTF-8 character; `i` only ever lands on a boundary because
        // every tag we skip is pure ASCII.
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}


/// One run of paragraph text: plain prose, or the text of an anchor along with
/// the index of its URL in the document's link list.
struct Segment {
    text: String,
    link: Option<usize>,
}

/// Split a paragraph into plain and anchor runs, registering each anchor's href.
fn split_anchors(paragraph: &str, links: &mut Vec<String>) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut last = 0;

    for caps in anchor_regex().captures_iter(paragraph) {
        let whole = caps.get(0).unwrap();
        let href = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let text = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        let before = strip_tags(&paragraph[last..whole.start()]);
        if !before.is_empty() {
            segments.push(Segment { text: before, link: None });
        }

        let label = strip_tags(text);
        if !href.is_empty() {
            // Same URL twice in a document shares one entry.
            let idx = match links.iter().position(|l| l == href) {
                Some(i) => i,
                None => {
                    links.push(href.to_string());
                    links.len() - 1
                }
            };
            // An anchor with no text (an image link, say) still needs
            // something to click on.
            let label = if label.is_empty() { href.to_string() } else { label };
            segments.push(Segment { text: label, link: Some(idx) });
        } else if !label.is_empty() {
            segments.push(Segment { text: label, link: None });
        }

        last = whole.end();
    }

    let tail = strip_tags(&paragraph[last..]);
    if !tail.is_empty() {
        segments.push(Segment { text: tail, link: None });
    }
    segments
}
/// Word-wrap a run of segments, keeping link runs as separate spans and
/// reporting the column range each one occupies on its line.
fn wrap_segments(
    segments: &[Segment],
    width: usize,
    theme: &Theme,
) -> Vec<(Line<'static>, Vec<(u16, u16, usize)>)> {
    let body = Style::default().fg(theme.reader_body);
    let link_style = Style::default()
        .fg(theme.reader_link_url)
        .add_modifier(Modifier::UNDERLINED);

    let mut out: Vec<(Line<'static>, Vec<(u16, u16, usize)>)> = Vec::new();
    // Runs on the line being built: (text, link index).
    let mut runs: Vec<(String, Option<usize>)> = Vec::new();
    let mut line_width = 0usize;

    let flush = |runs: &mut Vec<(String, Option<usize>)>,
                 out: &mut Vec<(Line<'static>, Vec<(u16, u16, usize)>)>| {
        if runs.is_empty() {
            return;
        }
        let mut spans = Vec::with_capacity(runs.len());
        let mut hits = Vec::new();
        let mut col = 0u16;
        for (text, link) in runs.drain(..) {
            let w = unicode_width::UnicodeWidthStr::width(text.as_str()) as u16;
            if let Some(idx) = link {
                hits.push((col, col + w, idx));
            }
            spans.push(Span::styled(text, if link.is_some() { link_style } else { body }));
            col += w;
        }
        out.push((Line::from(spans), hits));
    };

    for seg in segments {
        for word in seg.text.split_whitespace() {
            let word_width = unicode_width::UnicodeWidthStr::width(word);
            let gap = if line_width == 0 { 0 } else { 1 };

            if line_width > 0 && line_width + gap + word_width > width {
                flush(&mut runs, &mut out);
                line_width = 0;
            }

            // The separating space belongs to no link, so underlines stop at
            // the word boundary.
            if line_width > 0 {
                match runs.last_mut() {
                    Some((text, None)) => text.push(' '),
                    _ => runs.push((" ".to_string(), None)),
                }
                line_width += 1;
            }

            match runs.last_mut() {
                Some((text, link)) if *link == seg.link => text.push_str(word),
                _ => runs.push((word.to_string(), seg.link)),
            }
            line_width += word_width;
        }
    }

    flush(&mut runs, &mut out);
    out
}

fn strip_tags(input: &str) -> String {
    let stripped = tag_regex().replace_all(input, " ");
    space_regex().replace_all(&stripped, " ").trim().to_string()
}


pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    for raw_line in text.lines() {
        let mut current_line = String::new();
        // Width of `current_line`, tracked incrementally: re-measuring the
        // accumulated string per word made wrapping quadratic in line length.
        let mut current_len = 0usize;
        let mut had_word = false;

        for word in raw_line.split_whitespace() {
            had_word = true;
            let word_len = unicode_width::UnicodeWidthStr::width(word);

            if current_line.is_empty() {
                current_line.push_str(word);
                current_len = word_len;
            } else if current_len + 1 + word_len <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
                current_len += 1 + word_len;
            } else {
                lines.push(std::mem::take(&mut current_line));
                current_line.push_str(word);
                current_len = word_len;
            }
        }

        if !had_word {
            lines.push(String::new());
            continue;
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let prefix: String = s.chars().take(max_len.saturating_sub(1)).collect();
        format!("{prefix}…")
    }
}
