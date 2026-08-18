use crate::model::Article;
use crate::theme::Theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use regex::Regex;


#[allow(dead_code)]
pub struct FormattedArticle {

    pub lines: Vec<Line<'static>>,
    pub links: Vec<String>,
    pub total_lines: usize,
}

pub fn render_article_to_text(article: &Article, theme: &Theme, max_width: u16) -> FormattedArticle {
    let mut lines = Vec::new();
    let mut links = Vec::new();
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

    // Link URL line
    if !article.url.is_empty() {
        links.push(article.url.clone());
        lines.push(Line::from(vec![
            Span::styled("🔗 ", Style::default().fg(theme.reader_link)),
            Span::styled(
                truncate_string(&article.url, width.saturating_sub(4)),
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
    let raw_content = article
        .content
        .as_deref()
        .or(article.summary.as_deref())
        .unwrap_or("No content available for this article.");

    let body_lines = parse_html_to_ratatui_lines(raw_content, theme, width, &mut links);
    lines.extend(body_lines);

    // Add footer margin
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    let total_lines = lines.len();
    FormattedArticle {
        lines,
        links,
        total_lines,
    }
}

pub fn parse_html_to_ratatui_lines(
    html: &str,
    theme: &Theme,
    width: usize,
    links: &mut Vec<String>,
) -> Vec<Line<'static>> {
    let mut result = Vec::new();
    let unescaped = html_escape::decode_html_entities(html);

    // Split paragraphs/blocks roughly by block tags
    let normalized = unescaped
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n\n")
        .replace("</div>", "\n")
        .replace("</h1>", "\n\n")
        .replace("</h2>", "\n\n")
        .replace("</h3>", "\n\n")
        .replace("</h4>", "\n\n")
        .replace("</blockquote>", "\n\n")
        .replace("</li>", "\n")
        .replace("</ul>", "\n\n")
        .replace("</ol>", "\n\n")
        .replace("</pre>", "\n\n");

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

        // Standard Paragraph: parse spans for inline formatting (bold, italic, code, links)
        let clean_paragraph = strip_tags_and_extract_links(p_trimmed, links);
        let wrapped = wrap_text(&clean_paragraph, width);
        for w in wrapped {
            result.push(Line::from(vec![Span::styled(
                w,
                Style::default().fg(theme.reader_body),
            )]));
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

fn strip_tags(input: &str) -> String {
    let re = Regex::new(r"<[^>]*>").unwrap();
    let stripped = re.replace_all(input, " ");
    let re_spaces = Regex::new(r"\s+").unwrap();
    re_spaces.replace_all(&stripped, " ").trim().to_string()
}

fn strip_tags_and_extract_links(input: &str, links: &mut Vec<String>) -> String {
    // Extract hrefs from <a href="...">
    let re_a = Regex::new(r#"(?i)<a\s+[^>]*href=["']([^"']+)["'][^>]*>(.*?)</a>"#).unwrap();
    let transformed = re_a.replace_all(input, |caps: &regex::Captures| {
        let href = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let anchor_text = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if !href.is_empty() && !links.contains(&href.to_string()) {
            links.push(href.to_string());
        }
        anchor_text.to_string()
    });

    strip_tags(&transformed)
}

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    for raw_line in text.lines() {
        let words = raw_line.split_whitespace().collect::<Vec<_>>();
        if words.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        for word in words {
            let word_len = unicode_width::UnicodeWidthStr::width(word);
            let current_len = unicode_width::UnicodeWidthStr::width(current_line.as_str());

            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_len + 1 + word_len <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
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
