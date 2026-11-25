//! Helpers for Discord's text formatting: mentions, timestamps and markdown.
//!
//! Works with any Discord library. Everything produces plain `String`s.
//!
//! ```
//! use ds_format::prelude::*;
//!
//! let ends = Timestamp::parse("2026-11-25 18:00")?;
//! let msg = format!(
//!     "{} the event ends {} ({})",
//!     mention::role(1234),
//!     ends.relative(),
//!     ends.short_time().bold(),
//! );
//! assert_eq!(msg, "<@&1234> the event ends <t:1795629600:R> (**<t:1795629600:t>**)");
//! # Ok::<(), ds_format::FormatError>(())
//! ```
//!
//! # Features
//! - `tz`: re-exports [`chrono_tz`], so dates can be read in zones like
//!   `Europe::Bratislava` with correct summer time.

pub mod ext;
pub mod limits;
pub mod markdown;
pub mod mention;
pub mod timestamp;

pub use ext::Format;
pub use mention::{IntoId, Mention, Navigation};
pub use timestamp::{Style, Styled, Timestamp};

pub use chrono;
#[cfg(feature = "tz")]
pub use chrono_tz;

/// Everything needed to format a message: `use ds_format::prelude::*;`
///
/// Brings in the [`markdown`] functions, the [`mention`] module, the
/// [`Format`] trait and the timestamp types.
pub mod prelude {
    pub use crate::markdown::*;
    pub use crate::{Format, IntoId, Mention, Style, Timestamp, limits, mention};
}

/// Errors returned by this crate.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FormatError {
    /// The text is not one of the accepted date formats, or not a real date.
    #[error("invalid date: {0:?}")]
    InvalidDate(String),
    /// The local time happens twice in that zone (clocks went back).
    #[error("ambiguous local time (happens twice in this time zone): {0:?}")]
    AmbiguousLocalTime(String),
    /// The local time does not exist in that zone (clocks jumped forward).
    #[error("local time does not exist in this time zone: {0:?}")]
    NonexistentLocalTime(String),
    /// Not one of the style letters `t T d D f F R`.
    #[error("invalid timestamp style: {0:?}")]
    InvalidStyle(String),
    /// Not a mention like `<@123>`, `<#123>` or `<:name:123>`.
    #[error("invalid mention: {0:?}")]
    InvalidMention(String),
}
