//! Markdown as methods, for chaining instead of nesting.

use std::fmt::Display;

use crate::markdown;

/// The [`markdown`] helpers as methods on anything that implements [`Display`].
///
/// ```
/// use ds_format::{Format, Timestamp};
///
/// assert_eq!("*user*".escaped().bold(), r"**\*user\***");
/// assert_eq!(Timestamp::from_unix(60).relative().italic(), "*<t:60:R>*");
/// assert_eq!(42.code(), "`42`");
/// ```
pub trait Format: Display {
    /// See [`markdown::escape`].
    fn escaped(&self) -> String {
        markdown::escape(&self.to_string())
    }

    /// See [`markdown::bold`].
    fn bold(&self) -> String {
        markdown::bold(self)
    }

    /// See [`markdown::italic`].
    fn italic(&self) -> String {
        markdown::italic(self)
    }

    /// See [`markdown::bold_italic`].
    fn bold_italic(&self) -> String {
        markdown::bold_italic(self)
    }

    /// See [`markdown::underline`].
    fn underline(&self) -> String {
        markdown::underline(self)
    }

    /// See [`markdown::strike`].
    fn strike(&self) -> String {
        markdown::strike(self)
    }

    /// See [`markdown::spoiler`].
    fn spoiler(&self) -> String {
        markdown::spoiler(self)
    }

    /// See [`markdown::code`].
    fn code(&self) -> String {
        markdown::code(&self.to_string())
    }

    /// See [`markdown::quote`].
    fn quote(&self) -> String {
        markdown::quote(self)
    }

    /// See [`markdown::subtext`].
    fn subtext(&self) -> String {
        markdown::subtext(self)
    }
}

impl<T: Display + ?Sized> Format for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chains() {
        assert_eq!("x".bold().italic(), "***x***");
        assert_eq!(String::from("a`b").code(), "`` a`b ``");
        assert_eq!("a\nb".quote(), "> a\n> b");
        assert_eq!("x".spoiler().strike().underline(), "__~~||x||~~__");
        assert_eq!("x".bold_italic(), "***x***");
        assert_eq!("x".subtext(), "-# x");
        assert_eq!("<@1>".escaped(), r"\<@1\>");
    }
}
