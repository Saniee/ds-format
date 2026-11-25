//! Helpers for Discord's text formatting: mentions, timestamps and markdown.
//!
//! Works with any Discord library. Everything produces plain `String`s.
//!
//! ```
//! use ds_format::{markdown, mention, Timestamp};
//!
//! let ends = Timestamp::parse("2026-11-25 18:00")?;
//! let msg = format!(
//!     "{} the event ends {} ({})",
//!     mention::role(1234u64),
//!     ends.relative(),
//!     markdown::bold(&ends.short_time()),
//! );
//! assert_eq!(msg, "<@&1234> the event ends <t:1795629600:R> (**<t:1795629600:t>**)");
//! # Ok::<(), ds_format::FormatError>(())
//! ```
//!
//! # Features
//! - `tz`: re-exports [`chrono_tz`], so dates can be read in zones like
//!   `Europe::Bratislava` with correct summer time.

pub mod markdown;
pub mod mention;
pub mod timestamp;

pub use mention::Mention;
pub use timestamp::{Style, Timestamp};

pub use chrono;
#[cfg(feature = "tz")]
pub use chrono_tz;

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
}
