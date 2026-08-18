//! Subsequence fuzzy matching with fzf-style scoring.
//!
//! Written rather than pulled in as a dependency: the whole matcher is a couple
//! of linear passes, and it runs over every article title on every keystroke, so
//! it is worth keeping allocation-free and inlineable.

/// Score awarded for each matched character.
const BASE: i32 = 16;
/// Extra for a match that directly follows the previous one.
const CONSECUTIVE: i32 = 12;
/// Extra for a match at the start of a word.
const WORD_START: i32 = 10;
/// Extra for a match at the very start of the text.
const PREFIX: i32 = 8;
/// Charged per skipped character, so tighter matches rank higher.
const GAP: i32 = 1;

/// Is `haystack[idx]` the first character of a word?
fn is_word_start(chars: &[char], idx: usize) -> bool {
    if idx == 0 {
        return true;
    }
    let prev = chars[idx - 1];
    let cur = chars[idx];
    !prev.is_alphanumeric() || (prev.is_lowercase() && cur.is_uppercase())
}

/// Score one alignment, or `None` if the needle is not a subsequence.
///
/// `boundary_first` makes each character prefer the next word-start occurrence
/// over the next occurrence of any kind, which is what makes "bm" rank
/// "Bloomberg Markets" above a mid-word coincidence.
fn score_alignment(hay: &[char], needle: &[char], boundary_first: bool) -> Option<i32> {
    let mut score = 0;
    let mut hay_idx = 0usize;
    let mut prev_match: Option<usize> = None;

    for &want in needle {
        // Candidate positions from the current cursor onward.
        let mut chosen: Option<usize> = None;
        let mut first_any: Option<usize> = None;

        for (offset, &c) in hay[hay_idx..].iter().enumerate() {
            let at = hay_idx + offset;
            if c.to_lowercase().next() != Some(want) {
                continue;
            }
            if first_any.is_none() {
                first_any = Some(at);
            }
            // Taking a consecutive character is always at least as good as
            // hunting for a later boundary.
            if prev_match == Some(at.wrapping_sub(1)) {
                chosen = Some(at);
                break;
            }
            if !boundary_first {
                chosen = Some(at);
                break;
            }
            if is_word_start(hay, at) {
                chosen = Some(at);
                break;
            }
        }

        let at = chosen.or(first_any)?;

        score += BASE;
        if at == 0 {
            score += PREFIX;
        }
        if is_word_start(hay, at) {
            score += WORD_START;
        }
        match prev_match {
            Some(prev) if prev + 1 == at => score += CONSECUTIVE,
            Some(prev) => score -= GAP * (at - prev - 1) as i32,
            None => score -= GAP * at as i32,
        }

        prev_match = Some(at);
        hay_idx = at + 1;
    }

    Some(score)
}

/// Fuzzy-match `needle` (which must already be lowercase) against `haystack`.
///
/// Returns the match score, higher being better, or `None` when the needle's
/// characters do not appear in order. An empty needle matches everything.
pub fn score(haystack: &str, needle_lower: &str) -> Option<i32> {
    if needle_lower.is_empty() {
        return Some(0);
    }
    if haystack.is_empty() {
        return None;
    }

    let hay: Vec<char> = haystack.chars().collect();
    let needle: Vec<char> = needle_lower.chars().collect();
    if needle.len() > hay.len() {
        return None;
    }

    let greedy = score_alignment(&hay, &needle, false);
    // No point running the boundary pass if the needle isn't present at all.
    greedy?;
    let boundary = score_alignment(&hay, &needle, true);
    Some(greedy.unwrap_or(i32::MIN).max(boundary.unwrap_or(i32::MIN)))
}

/// Best score across several fields, e.g. an article's title and its feed.
pub fn score_any<'a>(fields: impl IntoIterator<Item = &'a str>, needle_lower: &str) -> Option<i32> {
    fields
        .into_iter()
        .filter_map(|f| score(f, needle_lower))
        .max()
}
