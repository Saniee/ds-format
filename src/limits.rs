//! Discord's length limits, and helpers to stay within them.
//!
//! Lengths are counted in characters (Unicode scalar values), not bytes.
//!
//! ```
//! use ds_format::limits::{self, split_message, truncate};
//!
//! assert_eq!(truncate("Very long title", 9), "Very lon…");
//! for part in split_message(&"word ".repeat(1000), limits::MESSAGE) {
//!     assert!(part.chars().count() <= limits::MESSAGE);
//! }
//! ```

use std::borrow::Cow;

/// Message content.
pub const MESSAGE: usize = 2000;
/// Embed title.
pub const EMBED_TITLE: usize = 256;
/// Embed description.
pub const EMBED_DESCRIPTION: usize = 4096;
/// Embed field name.
pub const FIELD_NAME: usize = 256;
/// Embed field value.
pub const FIELD_VALUE: usize = 1024;
/// Embed footer text.
pub const FOOTER: usize = 2048;
/// Embed author name.
pub const AUTHOR_NAME: usize = 256;
/// All text in all embeds of one message together.
pub const EMBED_TOTAL: usize = 6000;

/// Shortens `text` to at most `max` characters, ending in `…` when cut.
///
/// ```
/// use ds_format::limits::truncate;
/// assert_eq!(truncate("short", 10), "short");
/// assert_eq!(truncate("čučoriedky", 5), "čučo…");
/// ```
pub fn truncate(text: &str, max: usize) -> Cow<'_, str> {
    match text.char_indices().nth(max) {
        None => Cow::Borrowed(text),
        Some(_) if max == 0 => Cow::Borrowed(""),
        Some(_) => {
            let keep = text
                .char_indices()
                .nth(max - 1)
                .map_or(text.len(), |(i, _)| i);
            Cow::Owned(format!("{}…", &text[..keep]))
        }
    }
}

/// The smallest `max` that [`split_message`] accepts.
pub const MIN_SPLIT: usize = 32;

/// Longest code-block opening line (```` ```lang ````) reused when a block is split.
const MAX_FENCE_HEADER: usize = 16;
const FENCE_CLOSE: &str = "\n```";

/// Splits `text` into parts of at most `max` characters each, for sending as
/// several messages.
///
/// Cuts between lines where it can, otherwise between words, otherwise
/// anywhere. When a cut falls inside a ```` ``` ```` code block, the block is
/// closed at the end of the part and reopened (with the same language) at the
/// start of the next one. Text that already fits comes back as one part;
/// empty text gives no parts.
///
/// # Panics
/// If `max` is below [`MIN_SPLIT`].
pub fn split_message(text: &str, max: usize) -> Vec<String> {
    assert!(
        max >= MIN_SPLIT,
        "split_message: max must be at least {MIN_SPLIT}, got {max}"
    );
    if text.is_empty() {
        return Vec::new();
    }
    if text.chars().count() <= max {
        return vec![text.to_owned()];
    }
    let mut s = Splitter {
        max,
        parts: Vec::new(),
        cur: String::new(),
        cur_len: 0,
        cur_lines: 0,
        has_content: false,
        fence: None,
    };
    for line in text.split('\n') {
        s.push_line(line);
    }
    if s.has_content {
        s.parts.push(s.cur);
    }
    s.parts
}

struct Splitter {
    max: usize,
    parts: Vec<String>,
    /// The part being filled.
    cur: String,
    cur_len: usize,
    cur_lines: usize,
    /// Whether `cur` holds anything besides a reopened fence.
    has_content: bool,
    /// Opening line of the code block that is open at the end of `cur`.
    fence: Option<String>,
}

impl Splitter {
    fn push_line(&mut self, line: &str) {
        let toggles = line.matches("```").count() % 2 == 1;
        let after = match (&self.fence, toggles) {
            (fence, false) => fence.clone(),
            (Some(_), true) => None,
            (None, true) => Some(fence_header(line)),
        };
        let len = line.chars().count();

        if self.space(after.is_some()) >= len {
            self.append(line, len);
            self.fence = after;
            return;
        }
        if self.has_content {
            self.flush();
            if self.space(after.is_some()) >= len {
                self.append(line, len);
                self.fence = after;
                return;
            }
        }

        // Longer than a whole part: cut it into pieces, one part each.
        let fenced = self.fence.is_some() || after.is_some();
        let mut rest = line;
        loop {
            let (piece, next) = cut(rest, self.space(fenced));
            self.append(piece, piece.chars().count());
            if next.is_empty() {
                break;
            }
            self.flush();
            rest = next;
        }
        self.fence = after;
    }

    /// Characters left for the next line, keeping room to close an open code block.
    fn space(&self, fenced: bool) -> usize {
        let sep = usize::from(self.cur_lines > 0);
        let close = if fenced { FENCE_CLOSE.len() } else { 0 };
        self.max.saturating_sub(self.cur_len + sep + close)
    }

    fn append(&mut self, line: &str, len: usize) {
        if self.cur_lines > 0 {
            self.cur.push('\n');
            self.cur_len += 1;
        }
        self.cur.push_str(line);
        self.cur_len += len;
        self.cur_lines += 1;
        self.has_content = true;
    }

    fn flush(&mut self) {
        if self.fence.is_some() {
            self.cur.push_str(FENCE_CLOSE);
        }
        self.parts.push(std::mem::take(&mut self.cur));
        self.cur_len = 0;
        self.cur_lines = 0;
        self.has_content = false;
        if let Some(header) = &self.fence {
            self.cur.push_str(header);
            self.cur_len = header.chars().count();
            self.cur_lines = 1;
        }
    }
}

/// The line to reopen a code block with: the original ```` ```lang ```` if it is short and simple.
fn fence_header(line: &str) -> String {
    let t = line.trim();
    let simple = t.starts_with("```")
        && t.chars().count() <= MAX_FENCE_HEADER
        && !t[3..].contains(['`', ' ']);
    if simple {
        t.to_owned()
    } else {
        "```".to_owned()
    }
}

/// Splits off at most `avail` characters, preferring to cut at whitespace
/// (which is dropped). `avail` must be at least 1.
fn cut(text: &str, avail: usize) -> (&str, &str) {
    let Some((end, at_end)) = text.char_indices().nth(avail) else {
        return (text, "");
    };
    if at_end.is_whitespace() {
        return (&text[..end], &text[end + at_end.len_utf8()..]);
    }
    match text[..end]
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
    {
        Some((ws, c)) if ws > 0 => (&text[..ws], &text[ws + c.len_utf8()..]),
        _ => (&text[..end], &text[end..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn len(s: &str) -> usize {
        s.chars().count()
    }

    #[test]
    fn truncates_by_chars() {
        assert_eq!(truncate("abc", 3), "abc");
        assert!(matches!(truncate("abc", 3), Cow::Borrowed(_)));
        assert_eq!(truncate("abcd", 3), "ab…");
        assert_eq!(truncate("🙂🙂🙂", 2), "🙂…");
        assert_eq!(truncate("abc", 1), "…");
        assert_eq!(truncate("abc", 0), "");
        assert_eq!(truncate("", 0), "");
    }

    #[test]
    fn short_and_empty_text() {
        assert_eq!(split_message("hi", MESSAGE), vec!["hi"]);
        assert!(split_message("", MESSAGE).is_empty());
    }

    #[test]
    #[should_panic(expected = "at least 32")]
    fn rejects_tiny_max() {
        split_message("x", 5);
    }

    #[test]
    fn splits_between_lines() {
        let line = "x".repeat(20);
        let text = [line.as_str(); 3].join("\n");
        assert_eq!(
            split_message(&text, 45),
            vec![format!("{line}\n{line}"), line.clone()]
        );
    }

    #[test]
    fn splits_long_lines_between_words() {
        let text = "čučoriedka ".repeat(20);
        let parts = split_message(text.trim_end(), 40);
        assert!(parts.len() > 1);
        for p in &parts {
            assert!(len(p) <= 40, "{p:?}");
            assert!(!p.starts_with(' ') && !p.ends_with(' '), "{p:?}");
            assert!(p.split(' ').all(|w| w == "čučoriedka"), "{p:?}");
        }
    }

    #[test]
    fn hard_cuts_a_word_without_spaces() {
        let text = "🙂".repeat(100);
        let parts = split_message(&text, 32);
        assert!(parts.iter().all(|p| len(p) <= 32));
        assert_eq!(parts.concat(), text);
    }

    #[test]
    fn keeps_code_blocks_closed() {
        let body = ["let x = 1234567890;"; 6].join("\n");
        let text = format!("Intro\n```rs\n{body}\n```\nOutro");
        let parts = split_message(&text, 50);
        assert!(parts.len() > 2);
        for p in &parts {
            assert!(len(p) <= 50, "{p:?}");
            assert_eq!(p.matches("```").count() % 2, 0, "unbalanced: {p:?}");
        }
        assert!(
            parts[1..parts.len() - 1]
                .iter()
                .all(|p| p.starts_with("```rs\n"))
        );
        assert!(parts.last().unwrap().ends_with("Outro"));
    }

    #[test]
    fn preserves_text_when_cut_at_lines() {
        let text = (0..400)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let parts = split_message(&text, MESSAGE);
        assert!(parts.len() > 1);
        assert_eq!(parts.join("\n"), text);
    }
}
