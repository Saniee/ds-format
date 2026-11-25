//! Discord mention syntax: users, roles, channels, slash commands and emoji.
//!
//! ```
//! use ds_format::mention;
//!
//! assert_eq!(mention::user(80351110224678912), "<@80351110224678912>");
//! assert_eq!(format!("Hi {}!", mention::role(42)), "Hi <@&42>!");
//! ```
//!
//! Whether a mention actually pings someone is decided by the `allowed_mentions`
//! field of the message, not by this crate.

use std::fmt;
use std::num::NonZeroU64;
use std::str::FromStr;

use crate::FormatError;

/// A value usable as a Discord ID (snowflake).
///
/// Implemented for `u64` and `NonZeroU64` only, so a plain integer literal
/// like `mention::user(1234)` is inferred as `u64` without a suffix. Other
/// integer types convert with `u64::from(x)`; ID types of Discord libraries
/// usually have `.get()`.
pub trait IntoId {
    fn into_id(self) -> u64;
}

impl IntoId for u64 {
    fn into_id(self) -> u64 {
        self
    }
}

impl IntoId for NonZeroU64 {
    fn into_id(self) -> u64 {
        self.get()
    }
}

impl<T: IntoId + Copy> IntoId for &T {
    fn into_id(self) -> u64 {
        (*self).into_id()
    }
}

/// `<@id>`, mentions a user.
pub fn user(id: impl IntoId) -> String {
    Mention::User(id.into_id()).to_string()
}

/// `<@&id>`, mentions a role.
pub fn role(id: impl IntoId) -> String {
    Mention::Role(id.into_id()).to_string()
}

/// `<#id>`, links a channel or thread.
pub fn channel(id: impl IntoId) -> String {
    Mention::Channel(id.into_id()).to_string()
}

/// `</name:id>`, a clickable slash command.
///
/// For subcommands, pass the full path: `slash_command("config set", id)`.
///
/// ```
/// assert_eq!(ds_format::mention::slash_command("ping", 7), "</ping:7>");
/// ```
pub fn slash_command(name: &str, id: impl IntoId) -> String {
    Mention::SlashCommand {
        name: name.to_owned(),
        id: id.into_id(),
    }
    .to_string()
}

/// `<:name:id>` or `<a:name:id>` (animated), a custom emoji.
pub fn emoji(name: &str, id: impl IntoId, animated: bool) -> String {
    Mention::Emoji {
        name: name.to_owned(),
        id: id.into_id(),
        animated,
    }
    .to_string()
}

/// `@everyone`
pub fn everyone() -> &'static str {
    "@everyone"
}

/// `@here`
pub fn here() -> &'static str {
    "@here"
}

/// `<id:…>`, a link to one of the server's own pages.
///
/// ```
/// use ds_format::mention::{navigation, Navigation};
/// assert_eq!(navigation(Navigation::Customize), "<id:customize>");
/// ```
pub fn navigation(to: Navigation) -> String {
    Mention::Navigation(to).to_string()
}

/// `https://discord.com/channels/guild/channel/message`, a link that jumps to a message.
pub fn message_link(guild: impl IntoId, channel: impl IntoId, message: impl IntoId) -> String {
    format!(
        "https://discord.com/channels/{}/{}/{}",
        guild.into_id(),
        channel.into_id(),
        message.into_id()
    )
}

/// Server pages that `<id:…>` can link to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Navigation {
    /// `<id:customize>`: Channels & Roles
    Customize,
    /// `<id:browse>`: Browse Channels
    Browse,
    /// `<id:guide>`: Server Guide
    Guide,
    /// `<id:linked-roles>`: Linked Roles
    LinkedRoles,
}

impl Navigation {
    fn as_str(self) -> &'static str {
        match self {
            Navigation::Customize => "customize",
            Navigation::Browse => "browse",
            Navigation::Guide => "guide",
            Navigation::LinkedRoles => "linked-roles",
        }
    }
}

/// A mention as a value. `Display` writes the Discord syntax, `FromStr` reads it back.
///
/// ```
/// use ds_format::Mention;
/// assert_eq!("<@!42>".parse::<Mention>()?, Mention::User(42));
/// assert_eq!("<a:blob:7>".parse::<Mention>()?.to_string(), "<a:blob:7>");
/// # Ok::<(), ds_format::FormatError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Mention {
    User(u64),
    Role(u64),
    Channel(u64),
    SlashCommand {
        name: String,
        id: u64,
    },
    Emoji {
        name: String,
        id: u64,
        animated: bool,
    },
    Everyone,
    Here,
    Navigation(Navigation),
}

impl fmt::Display for Mention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mention::User(id) => write!(f, "<@{id}>"),
            Mention::Role(id) => write!(f, "<@&{id}>"),
            Mention::Channel(id) => write!(f, "<#{id}>"),
            Mention::SlashCommand { name, id } => write!(f, "</{}:{id}>", name.trim()),
            Mention::Emoji { name, id, animated } => {
                let a = if *animated { "a" } else { "" };
                write!(f, "<{a}:{name}:{id}>")
            }
            Mention::Everyone => f.write_str("@everyone"),
            Mention::Here => f.write_str("@here"),
            Mention::Navigation(to) => write!(f, "<id:{}>", to.as_str()),
        }
    }
}

impl FromStr for Mention {
    type Err = FormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || FormatError::InvalidMention(s.to_owned());
        let id = |text: &str| text.parse::<u64>().map_err(|_| invalid());
        let named = |text: &str| {
            let (name, raw_id) = text.rsplit_once(':').ok_or_else(invalid)?;
            if name.is_empty() {
                return Err(invalid());
            }
            Ok((name.to_owned(), id(raw_id)?))
        };

        let s_trim = s.trim();
        match s_trim {
            "@everyone" => return Ok(Mention::Everyone),
            "@here" => return Ok(Mention::Here),
            _ => {}
        }
        let inner = s_trim
            .strip_prefix('<')
            .and_then(|t| t.strip_suffix('>'))
            .ok_or_else(invalid)?;

        if let Some(rest) = inner.strip_prefix("id:") {
            let to = match rest {
                "customize" => Navigation::Customize,
                "browse" => Navigation::Browse,
                "guide" => Navigation::Guide,
                "linked-roles" => Navigation::LinkedRoles,
                _ => return Err(invalid()),
            };
            return Ok(Mention::Navigation(to));
        }
        if let Some(rest) = inner.strip_prefix("@&") {
            return Ok(Mention::Role(id(rest)?));
        }
        if let Some(rest) = inner.strip_prefix("@!").or_else(|| inner.strip_prefix('@')) {
            return Ok(Mention::User(id(rest)?));
        }
        if let Some(rest) = inner.strip_prefix('#') {
            return Ok(Mention::Channel(id(rest)?));
        }
        if let Some(rest) = inner.strip_prefix('/') {
            let (name, id) = named(rest)?;
            return Ok(Mention::SlashCommand { name, id });
        }
        let (animated, rest) = match inner.strip_prefix("a:") {
            Some(rest) => (true, rest),
            None => (false, inner.strip_prefix(':').ok_or_else(invalid)?),
        };
        let (name, id) = named(rest)?;
        if name.contains(':') {
            return Err(invalid());
        }
        Ok(Mention::Emoji { name, id, animated })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_forms() {
        assert_eq!(user(1), "<@1>");
        assert_eq!(role(2), "<@&2>");
        assert_eq!(channel(3), "<#3>");
        assert_eq!(slash_command("ping", 4), "</ping:4>");
        assert_eq!(slash_command(" config set ", 5), "</config set:5>");
        assert_eq!(emoji("blob", 6, false), "<:blob:6>");
        assert_eq!(emoji("blob", 7, true), "<a:blob:7>");
        assert_eq!(everyone(), "@everyone");
        assert_eq!(here(), "@here");
        assert_eq!(navigation(Navigation::LinkedRoles), "<id:linked-roles>");
        assert_eq!(Mention::Everyone.to_string(), "@everyone");
        assert_eq!(Mention::Here.to_string(), "@here");
    }

    #[test]
    fn ids_without_suffix() {
        assert_eq!(user(80351110224678912), "<@80351110224678912>");
        let id: u64 = 9;
        assert_eq!(user(id), "<@9>");
        let ids = [id];
        assert_eq!(ids.iter().map(user).collect::<Vec<_>>(), ["<@9>"]);
        assert_eq!(user(NonZeroU64::new(9).unwrap()), "<@9>");
        assert_eq!(user(u64::from(9u32)), "<@9>");
    }

    #[test]
    fn message_links() {
        assert_eq!(message_link(1, 2, 3), "https://discord.com/channels/1/2/3");
    }

    #[test]
    fn parses_what_it_writes() {
        let all = [
            Mention::User(1),
            Mention::Role(2),
            Mention::Channel(3),
            Mention::SlashCommand {
                name: "config set".into(),
                id: 4,
            },
            Mention::Emoji {
                name: "blob".into(),
                id: 5,
                animated: false,
            },
            Mention::Emoji {
                name: "blob".into(),
                id: 6,
                animated: true,
            },
            Mention::Everyone,
            Mention::Here,
            Mention::Navigation(Navigation::Customize),
            Mention::Navigation(Navigation::Browse),
            Mention::Navigation(Navigation::Guide),
            Mention::Navigation(Navigation::LinkedRoles),
        ];
        for m in all {
            assert_eq!(m.to_string().parse::<Mention>().unwrap(), m);
        }
        assert_eq!(" <@!7> ".parse::<Mention>().unwrap(), Mention::User(7));
    }

    #[test]
    fn rejects_bad_mentions() {
        for bad in [
            "",
            "<>",
            "<@>",
            "<@x>",
            "<@-1>",
            "@1",
            "<#1",
            "</:1>",
            "</ping>",
            "<:x>",
            "<::1>",
            "<a:b:c:1>",
            "<id:nope>",
            "everyone",
        ] {
            assert!(
                matches!(bad.parse::<Mention>(), Err(FormatError::InvalidMention(_))),
                "{bad:?} should fail"
            );
        }
    }
}
