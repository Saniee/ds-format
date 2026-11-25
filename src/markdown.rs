//! Discord markdown helpers.
//!
//! ```
//! use ds_format::markdown::{bold, escape};
//!
//! let name = "*not bold*";
//! assert_eq!(bold(escape(name)), r"**\*not bold\***");
//! ```
//!
//! The wrappers take anything that implements [`Display`], so the output of
//! other helpers can be passed straight in: `bold(ts.relative())`.
//!
//! None of these escape their input. Run user-supplied text through
//! [`escape`] first.

use std::fmt::Display;

/// Characters that Discord treats as markdown.
const SPECIAL: &[char] = &[
    '\\', '*', '_', '~', '|', '`', '>', '<', '#', '-', '[', ']', '(', ')',
];

/// Escapes markdown characters so text shows up literally.
///
/// Also escapes `<` (so pasted `<@id>` or `<t:…>` stay text) and the dot of a
/// numbered list item at the start of a line (`1. ` → `1\. `).
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let list_dot = numbered_list_dot(line);
        for (pos, c) in line.char_indices() {
            if SPECIAL.contains(&c) || Some(pos) == list_dot {
                out.push('\\');
            }
            out.push(c);
        }
    }
    out
}

/// Byte index of the `.` in a line like `  12. item`, which Discord renders as a list.
fn numbered_list_dot(line: &str) -> Option<usize> {
    let trimmed = line.trim_start_matches(' ');
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    (digits > 0 && trimmed[digits..].starts_with(". "))
        .then_some(line.len() - trimmed.len() + digits)
}

/// Puts a backslash before each of `chars` that is not escaped already.
fn escape_unescaped(text: &str, chars: &[char]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut escaped = false;
    for c in text.chars() {
        if !escaped && chars.contains(&c) {
            out.push('\\');
        }
        escaped = !escaped && c == '\\';
        out.push(c);
    }
    out
}

/// `**text**`
pub fn bold(text: impl Display) -> String {
    format!("**{text}**")
}

/// `*text*`
pub fn italic(text: impl Display) -> String {
    format!("*{text}*")
}

/// `***text***`
pub fn bold_italic(text: impl Display) -> String {
    format!("***{text}***")
}

/// `__text__`
pub fn underline(text: impl Display) -> String {
    format!("__{text}__")
}

/// `~~text~~`
pub fn strike(text: impl Display) -> String {
    format!("~~{text}~~")
}

/// `||text||`
pub fn spoiler(text: impl Display) -> String {
    format!("||{text}||")
}

/// Inline code. Uses double backticks if the text contains a backtick.
pub fn code(text: &str) -> String {
    if text.contains('`') {
        format!("`` {text} ``")
    } else {
        format!("`{text}`")
    }
}

/// A fenced code block. `lang` may be empty.
///
/// A ```` ``` ```` inside `text` gets a zero-width space between its backticks
/// so it can't end the block early.
///
/// ```
/// assert_eq!(ds_format::markdown::code_block("rs", "let x = 1;"), "```rs\nlet x = 1;\n```");
/// ```
pub fn code_block(lang: &str, text: &str) -> String {
    let text = text
        .trim_end_matches('\n')
        .replace("```", "`\u{200B}`\u{200B}`");
    format!("```{}\n{text}\n```", lang.trim())
}

/// Quotes every line with `> `.
pub fn quote(text: impl Display) -> String {
    let text = text.to_string();
    if text.is_empty() {
        return "> ".to_owned();
    }
    text.lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `>>> text`: quotes the text and everything after it in the message.
pub fn block_quote(text: impl Display) -> String {
    format!(">>> {text}")
}

/// `#`, `##` or `###` header. Levels outside 1..=3 are clamped.
pub fn header(level: u8, text: impl Display) -> String {
    format!("{} {text}", "#".repeat(level.clamp(1, 3) as usize))
}

/// `-# text`, small grey subtext.
pub fn subtext(text: impl Display) -> String {
    format!("-# {text}")
}

/// A bulleted list, one `- item` per line.
///
/// ```
/// assert_eq!(ds_format::markdown::list(["a", "b"]), "- a\n- b");
/// ```
pub fn list<I>(items: I) -> String
where
    I: IntoIterator,
    I::Item: Display,
{
    items
        .into_iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A numbered list starting at 1.
///
/// ```
/// assert_eq!(ds_format::markdown::numbered_list(["a", "b"]), "1. a\n2. b");
/// ```
pub fn numbered_list<I>(items: I) -> String
where
    I: IntoIterator,
    I::Item: Display,
{
    items
        .into_iter()
        .enumerate()
        .map(|(i, item)| format!("{}. {item}", i + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `<url>`, a link that doesn't show an embed preview.
pub fn link(url: &str) -> String {
    format!("<{url}>")
}

/// `[text](url)`
///
/// `[` and `]` in the text are escaped (unless already escaped) and `)` in
/// the URL becomes `%29`, so neither can break the link.
pub fn masked_link(text: impl Display, url: &str) -> String {
    format!("[{}]({})", link_text(text), link_url(url))
}

/// `[text](<url>)`, a masked link that doesn't show an embed preview.
pub fn masked_link_no_embed(text: impl Display, url: &str) -> String {
    format!("[{}](<{}>)", link_text(text), link_url(url))
}

fn link_text(text: impl Display) -> String {
    escape_unescaped(&text.to_string(), &['[', ']'])
}

fn link_url(url: &str) -> String {
    url.trim().replace(')', "%29")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_everything_special() {
        assert_eq!(
            escape(r"a*b_c~d|e`f>g#h-i[j]k(l)m\n<o"),
            r"a\*b\_c\~d\|e\`f\>g\#h\-i\[j\]k\(l\)m\\n\<o"
        );
        assert_eq!(escape("plain text 123"), "plain text 123");
        assert_eq!(escape("čšž 🙂"), "čšž 🙂");
        assert_eq!(escape("<@1>"), r"\<@1\>");
    }

    #[test]
    fn escapes_numbered_lists_only_at_line_start() {
        assert_eq!(escape("1. a\n  22. b"), "1\\. a\n  22\\. b");
        assert_eq!(escape("x 1. a"), "x 1. a");
        assert_eq!(escape("1.5 kg"), "1.5 kg");
        assert_eq!(escape("3."), "3.");
    }

    #[test]
    fn wrappers() {
        assert_eq!(bold("x"), "**x**");
        assert_eq!(italic("x"), "*x*");
        assert_eq!(bold_italic("x"), "***x***");
        assert_eq!(underline("x"), "__x__");
        assert_eq!(strike("x"), "~~x~~");
        assert_eq!(spoiler("x"), "||x||");
        assert_eq!(code("x"), "`x`");
        assert_eq!(code("a`b"), "`` a`b ``");
        assert_eq!(code_block("", "x\n"), "```\nx\n```");
        assert_eq!(quote("a\nb"), "> a\n> b");
        assert_eq!(quote(""), "> ");
        assert_eq!(block_quote("a\nb"), ">>> a\nb");
        assert_eq!(header(0, "x"), "# x");
        assert_eq!(header(2, "x"), "## x");
        assert_eq!(header(9, "x"), "### x");
        assert_eq!(subtext("x"), "-# x");
        assert_eq!(masked_link("docs", "https://a.b"), "[docs](https://a.b)");
    }

    #[test]
    fn wrappers_take_any_display() {
        let owned = String::from("x");
        assert_eq!(bold(&owned), "**x**");
        assert_eq!(bold(owned), "**x**");
        assert_eq!(italic(42), "*42*");
        assert_eq!(bold(italic("x")), "***x***");
    }

    #[test]
    fn code_block_survives_fences_in_text() {
        let block = code_block("md", "```rs\nx\n```");
        assert_eq!(block.matches("```").count(), 2);
        assert!(block.starts_with("```md\n") && block.ends_with("\n```"));
    }

    #[test]
    fn lists() {
        assert_eq!(list(["a", "b"]), "- a\n- b");
        assert_eq!(list(Vec::<String>::new()), "");
        assert_eq!(numbered_list([1, 2, 3]), "1. 1\n2. 2\n3. 3");
    }

    #[test]
    fn links() {
        assert_eq!(link("https://a.b"), "<https://a.b>");
        assert_eq!(
            masked_link("[x]", "https://a.b/(c)"),
            r"[\[x\]](https://a.b/(c%29)"
        );
        assert_eq!(masked_link(escape("[x]"), "u"), r"[\[x\]](u)");
        assert_eq!(
            masked_link_no_embed("docs", "https://a.b"),
            "[docs](<https://a.b>)"
        );
    }
}
