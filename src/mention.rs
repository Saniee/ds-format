//! Discord mention syntax: users, roles, channels, slash commands and emoji.
//!
//! ```
//! use ds_format::mention;
//!
//! assert_eq!(mention::user(80351110224678912u64), "<@80351110224678912>");
//! assert_eq!(format!("Hi {}!", mention::role(42u64)), "Hi <@&42>!");
//! ```
//!
//! Whether a mention actually pings someone is decided by the `allowed_mentions`
//! field of the message, not by this crate.

use std::fmt;

/// `<@id>`, mentions a user.
pub fn user(id: impl Into<u64>) -> String {
    Mention::User(id.into()).to_string()
}

/// `<@&id>`, mentions a role.
pub fn role(id: impl Into<u64>) -> String {
    Mention::Role(id.into()).to_string()
}

/// `<#id>`, links a channel or thread.
pub fn channel(id: impl Into<u64>) -> String {
    Mention::Channel(id.into()).to_string()
}

/// `</name:id>`, a clickable slash command.
///
/// For subcommands, pass the full path: `slash_command("config set", id)`.
///
/// ```
/// assert_eq!(ds_format::mention::slash_command("ping", 7u64), "</ping:7>");
/// ```
pub fn slash_command(name: &str, id: impl Into<u64>) -> String {
    Mention::SlashCommand {
        name: name.to_owned(),
        id: id.into(),
    }
    .to_string()
}

/// `<:name:id>` or `<a:name:id>` (animated), a custom emoji.
pub fn emoji(name: &str, id: impl Into<u64>, animated: bool) -> String {
    Mention::Emoji {
        name: name.to_owned(),
        id: id.into(),
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

/// A mention as a value. `Display` writes the Discord syntax.
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_forms() {
        assert_eq!(user(1u64), "<@1>");
        assert_eq!(role(2u64), "<@&2>");
        assert_eq!(channel(3u64), "<#3>");
        assert_eq!(slash_command("ping", 4u64), "</ping:4>");
        assert_eq!(slash_command(" config set ", 5u64), "</config set:5>");
        assert_eq!(emoji("blob", 6u64, false), "<:blob:6>");
        assert_eq!(emoji("blob", 7u64, true), "<a:blob:7>");
        assert_eq!(everyone(), "@everyone");
        assert_eq!(here(), "@here");
        assert_eq!(Mention::Everyone.to_string(), "@everyone");
        assert_eq!(Mention::Here.to_string(), "@here");
    }

    #[test]
    fn accepts_smaller_int_types() {
        assert_eq!(user(9u32), "<@9>");
    }
}
