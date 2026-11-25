//! Discord markdown helpers.
//!
//! ```
//! use ds_format::markdown::{bold, escape};
//!
//! let name = "*not bold*";
//! assert_eq!(bold(&escape(name)), r"**\*not bold\***");
//! ```
//!
//! None of these escape their input. Run user-supplied text through
//! [`escape`] first.

/// Characters that Discord treats as markdown.
const SPECIAL: &[char] = &[
    '\\', '*', '_', '~', '|', '`', '>', '#', '-', '[', ']', '(', ')',
];

/// Escapes markdown characters so text shows up literally.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if SPECIAL.contains(&c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// `**text**`
pub fn bold(text: &str) -> String {
    format!("**{text}**")
}

/// `*text*`
pub fn italic(text: &str) -> String {
    format!("*{text}*")
}

/// `__text__`
pub fn underline(text: &str) -> String {
    format!("__{text}__")
}

/// `~~text~~`
pub fn strike(text: &str) -> String {
    format!("~~{text}~~")
}

/// `||text||`
pub fn spoiler(text: &str) -> String {
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
/// ```
/// assert_eq!(ds_format::markdown::code_block("rs", "let x = 1;"), "```rs\nlet x = 1;\n```");
/// ```
pub fn code_block(lang: &str, text: &str) -> String {
    format!("```{lang}\n{}\n```", text.trim_end_matches('\n'))
}

/// Quotes every line with `> `.
pub fn quote(text: &str) -> String {
    text.lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `#`, `##` or `###` header. Levels outside 1..=3 are clamped.
pub fn header(level: u8, text: &str) -> String {
    format!("{} {text}", "#".repeat(level.clamp(1, 3) as usize))
}

/// `-# text`, small grey subtext.
pub fn subtext(text: &str) -> String {
    format!("-# {text}")
}

/// `[text](url)`
pub fn masked_link(text: &str, url: &str) -> String {
    format!("[{text}]({url})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_everything_special() {
        assert_eq!(
            escape(r"a*b_c~d|e`f>g#h-i[j]k(l)m\n"),
            r"a\*b\_c\~d\|e\`f\>g\#h\-i\[j\]k\(l\)m\\n"
        );
        assert_eq!(escape("plain text 123"), "plain text 123");
        assert_eq!(escape("čšž 🙂"), "čšž 🙂");
    }

    #[test]
    fn wrappers() {
        assert_eq!(bold("x"), "**x**");
        assert_eq!(italic("x"), "*x*");
        assert_eq!(underline("x"), "__x__");
        assert_eq!(strike("x"), "~~x~~");
        assert_eq!(spoiler("x"), "||x||");
        assert_eq!(code("x"), "`x`");
        assert_eq!(code("a`b"), "`` a`b ``");
        assert_eq!(code_block("", "x\n"), "```\nx\n```");
        assert_eq!(quote("a\nb"), "> a\n> b");
        assert_eq!(header(0, "x"), "# x");
        assert_eq!(header(2, "x"), "## x");
        assert_eq!(header(9, "x"), "### x");
        assert_eq!(subtext("x"), "-# x");
        assert_eq!(masked_link("docs", "https://a.b"), "[docs](https://a.b)");
    }
}
