//! Discord timestamps (`<t:unix:style>`), which every viewer sees in their own time zone.
//!
//! ```
//! use ds_format::{Style, Timestamp};
//!
//! let ts = Timestamp::parse("2026-11-25 14:30")?; // read as UTC
//! assert_eq!(ts.relative(), "<t:1795617000:R>");
//! assert_eq!(ts.format(Style::LongDate), "<t:1795617000:D>");
//! # Ok::<(), ds_format::FormatError>(())
//! ```

use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::SystemTime;

use chrono::{
    DateTime, Duration, FixedOffset, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc,
};

use crate::FormatError;
use crate::mention::IntoId;

/// Start of Discord's snowflake clock (2015-01-01 00:00:00 UTC), in unix milliseconds.
const DISCORD_EPOCH_MS: u64 = 1_420_070_400_000;

/// How Discord displays a timestamp. Examples are for 2026-11-25 14:30 in en-GB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Style {
    /// `t`: 14:30
    ShortTime,
    /// `T`: 14:30:00
    LongTime,
    /// `d`: 25/11/2026
    ShortDate,
    /// `D`: 25 November 2026
    LongDate,
    /// `f`: 25 November 2026 14:30 (Discord's default)
    #[default]
    ShortDateTime,
    /// `F`: Wednesday, 25 November 2026 14:30
    LongDateTime,
    /// `R`: in 2 hours / 3 days ago
    Relative,
}

impl Style {
    /// The style letter Discord uses.
    pub fn letter(self) -> char {
        match self {
            Style::ShortTime => 't',
            Style::LongTime => 'T',
            Style::ShortDate => 'd',
            Style::LongDate => 'D',
            Style::ShortDateTime => 'f',
            Style::LongDateTime => 'F',
            Style::Relative => 'R',
        }
    }
}

impl FromStr for Style {
    type Err = FormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "t" => Style::ShortTime,
            "T" => Style::LongTime,
            "d" => Style::ShortDate,
            "D" => Style::LongDate,
            "f" => Style::ShortDateTime,
            "F" => Style::LongDateTime,
            "R" => Style::Relative,
            _ => return Err(FormatError::InvalidStyle(s.to_owned())),
        })
    }
}

/// A point in time, stored as unix seconds.
///
/// `Display` writes `<t:unix>`, which Discord shows in its default style (`f`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    unix: i64,
}

impl Timestamp {
    pub fn from_unix(unix: i64) -> Self {
        Self { unix }
    }

    pub fn now() -> Self {
        Utc::now().into()
    }

    pub fn unix(self) -> i64 {
        self.unix
    }

    /// When a Discord ID (user, message, channel, …) was created.
    ///
    /// ```
    /// use ds_format::Timestamp;
    /// // Example snowflake from Discord's API docs: 2016-04-30 11:18:25 UTC.
    /// assert_eq!(Timestamp::from_snowflake(175928847299117063).unix(), 1462015105);
    /// ```
    pub fn from_snowflake(id: impl IntoId) -> Self {
        let ms = (id.into_id() >> 22) + DISCORD_EPOCH_MS;
        // At most (2^42 + epoch) / 1000, which always fits in an i64.
        Self::from_unix((ms / 1000) as i64)
    }

    /// Parses a date, reading it as UTC if it has no offset of its own.
    ///
    /// Accepted forms:
    /// - `2026-11-25`: midnight
    /// - `2026-11-25 14:30`, `2026-11-25 14:30:15`, optionally with fractional seconds
    /// - `T` instead of the space
    /// - any of the above followed by `Z`, `+01:00` or `+0100`, an offset that always wins
    pub fn parse(input: &str) -> Result<Self, FormatError> {
        Self::parse_in(input, &Utc)
    }

    /// Like [`parse`](Self::parse), but reads dates without an offset in `zone`.
    ///
    /// `zone` can be `Utc`, `chrono::Local`, a `FixedOffset`, or with the `tz`
    /// feature a `chrono_tz` zone such as `Europe::Bratislava` (handles summer time).
    ///
    /// ```
    /// use ds_format::Timestamp;
    /// use chrono::FixedOffset;
    ///
    /// let cet = FixedOffset::east_opt(3600).unwrap();
    /// assert_eq!(Timestamp::parse_in("2026-11-25 14:30", &cet)?.unix(), 1795613400);
    /// # Ok::<(), ds_format::FormatError>(())
    /// ```
    pub fn parse_in<Z: TimeZone>(input: &str, zone: &Z) -> Result<Self, FormatError> {
        let normalized = normalize(input);

        if let Some(dt) = parse_with_offset(&normalized) {
            return Ok(dt.into());
        }

        let naive = parse_naive(&normalized)
            .ok_or_else(|| FormatError::InvalidDate(input.trim().to_owned()))?;
        match zone.from_local_datetime(&naive) {
            LocalResult::Single(dt) => Ok(dt.into()),
            LocalResult::Ambiguous(..) => {
                Err(FormatError::AmbiguousLocalTime(input.trim().to_owned()))
            }
            LocalResult::None => Err(FormatError::NonexistentLocalTime(input.trim().to_owned())),
        }
    }

    pub fn plus_seconds(self, secs: i64) -> Self {
        Self::from_unix(self.unix.saturating_add(secs))
    }

    pub fn plus_minutes(self, minutes: i64) -> Self {
        self.plus_seconds(minutes.saturating_mul(60))
    }

    pub fn plus_hours(self, hours: i64) -> Self {
        self.plus_seconds(hours.saturating_mul(3600))
    }

    pub fn plus_days(self, days: i64) -> Self {
        self.plus_seconds(days.saturating_mul(86_400))
    }

    pub fn plus_weeks(self, weeks: i64) -> Self {
        self.plus_seconds(weeks.saturating_mul(604_800))
    }

    /// `<t:unix:style>`
    pub fn format(self, style: Style) -> String {
        self.styled(style).to_string()
    }

    /// Like [`format`](Self::format), but returns a value that writes itself
    /// when displayed, without building a `String` first.
    ///
    /// ```
    /// use ds_format::{Style, Timestamp};
    /// let ts = Timestamp::from_unix(60);
    /// assert_eq!(format!("ends {}", ts.styled(Style::Relative)), "ends <t:60:R>");
    /// ```
    pub fn styled(self, style: Style) -> Styled {
        Styled { ts: self, style }
    }

    /// `<t:unix:t>`: 14:30
    pub fn short_time(self) -> String {
        self.format(Style::ShortTime)
    }

    /// `<t:unix:T>`: 14:30:00
    pub fn long_time(self) -> String {
        self.format(Style::LongTime)
    }

    /// `<t:unix:d>`: 25/11/2026
    pub fn short_date(self) -> String {
        self.format(Style::ShortDate)
    }

    /// `<t:unix:D>`: 25 November 2026
    pub fn long_date(self) -> String {
        self.format(Style::LongDate)
    }

    /// `<t:unix:f>`: 25 November 2026 14:30
    pub fn short_date_time(self) -> String {
        self.format(Style::ShortDateTime)
    }

    /// `<t:unix:F>`: Wednesday, 25 November 2026 14:30
    pub fn long_date_time(self) -> String {
        self.format(Style::LongDateTime)
    }

    /// `<t:unix:R>`: in 2 hours
    pub fn relative(self) -> String {
        self.format(Style::Relative)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<t:{}>", self.unix)
    }
}

/// A [`Timestamp`] with a [`Style`], from [`Timestamp::styled`]. `Display` writes `<t:unix:style>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Styled {
    ts: Timestamp,
    style: Style,
}

impl fmt::Display for Styled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<t:{}:{}>", self.ts.unix, self.style.letter())
    }
}

/// Adds whole seconds; fractions of a second are dropped.
impl Add<Duration> for Timestamp {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self {
        self.plus_seconds(rhs.num_seconds())
    }
}

/// Subtracts whole seconds; fractions of a second are dropped.
impl Sub<Duration> for Timestamp {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self {
        self.plus_seconds(rhs.num_seconds().saturating_neg())
    }
}

impl From<SystemTime> for Timestamp {
    fn from(t: SystemTime) -> Self {
        DateTime::<Utc>::from(t).into()
    }
}

impl<Z: TimeZone> From<DateTime<Z>> for Timestamp {
    fn from(dt: DateTime<Z>) -> Self {
        Self::from_unix(dt.timestamp())
    }
}

/// Reads the naive date time as UTC.
impl From<NaiveDateTime> for Timestamp {
    fn from(dt: NaiveDateTime) -> Self {
        Self::from_unix(dt.and_utc().timestamp())
    }
}

impl FromStr for Timestamp {
    type Err = FormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Trims, turns a `T` separator into a space and a trailing `Z` into `+00:00`.
fn normalize(input: &str) -> String {
    let mut s = input.trim().to_owned();
    if s.len() > 10 && matches!(s.as_bytes()[10], b'T' | b't') && s.is_char_boundary(11) {
        s.replace_range(10..11, " ");
    }
    if s.ends_with(['Z', 'z']) {
        s.pop();
        s.push_str("+00:00");
    }
    s
}

fn parse_with_offset(s: &str) -> Option<DateTime<FixedOffset>> {
    const FORMATS: &[&str] = &[
        "%Y-%m-%d %H:%M:%S%.f%:z",
        "%Y-%m-%d %H:%M:%S%.f%z",
        "%Y-%m-%d %H:%M%:z",
        "%Y-%m-%d %H:%M%z",
    ];
    FORMATS
        .iter()
        .find_map(|f| DateTime::parse_from_str(s, f).ok())
}

fn parse_naive(s: &str) -> Option<NaiveDateTime> {
    const FORMATS: &[&str] = &["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%d %H:%M"];
    FORMATS
        .iter()
        .find_map(|f| NaiveDateTime::parse_from_str(s, f).ok())
        .or_else(|| {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-11-25 14:30:00 UTC
    const BASE: i64 = 1_795_617_000;

    fn unix(s: &str) -> i64 {
        Timestamp::parse(s).unwrap().unix()
    }

    #[test]
    fn parses_naive_forms_as_utc() {
        assert_eq!(unix("2026-11-25 14:30"), BASE);
        assert_eq!(unix("2026-11-25T14:30"), BASE);
        assert_eq!(unix("  2026-11-25 14:30  "), BASE);
        assert_eq!(unix("2026-11-25 14:30:15"), BASE + 15);
        assert_eq!(unix("2026-11-25 14:30:15.750"), BASE + 15);
        assert_eq!(unix("2026-11-25"), 1_795_564_800);
    }

    #[test]
    fn own_offset_wins() {
        assert_eq!(unix("2026-11-25T14:30:00Z"), BASE);
        assert_eq!(unix("2026-11-25T14:30Z"), BASE);
        assert_eq!(unix("2026-11-25 14:30+01:00"), BASE - 3600);
        assert_eq!(unix("2026-11-25T14:30:00+0100"), BASE - 3600);
        let cet = FixedOffset::east_opt(3600).unwrap();
        assert_eq!(
            Timestamp::parse_in("2026-11-25 14:30Z", &cet)
                .unwrap()
                .unix(),
            BASE
        );
    }

    #[test]
    fn fixed_offset_zone() {
        let cet = FixedOffset::east_opt(3600).unwrap();
        assert_eq!(
            Timestamp::parse_in("2026-11-25 14:30", &cet)
                .unwrap()
                .unix(),
            BASE - 3600
        );
    }

    #[test]
    fn rejects_bad_input() {
        for bad in [
            "",
            "tomorrow",
            "2026-13-01",
            "2026-02-30 10:00",
            "25.11.2026",
            "2026-11-25 25:00",
        ] {
            assert!(
                matches!(Timestamp::parse(bad), Err(FormatError::InvalidDate(_))),
                "{bad:?} should fail"
            );
        }
    }

    #[test]
    fn styles() {
        let ts = Timestamp::from_unix(BASE);
        assert_eq!(ts.short_time(), "<t:1795617000:t>");
        assert_eq!(ts.long_time(), "<t:1795617000:T>");
        assert_eq!(ts.short_date(), "<t:1795617000:d>");
        assert_eq!(ts.long_date(), "<t:1795617000:D>");
        assert_eq!(ts.short_date_time(), "<t:1795617000:f>");
        assert_eq!(ts.long_date_time(), "<t:1795617000:F>");
        assert_eq!(ts.relative(), "<t:1795617000:R>");
        assert_eq!(ts.to_string(), "<t:1795617000>");
        assert_eq!(ts.styled(Style::Relative).to_string(), ts.relative());
        for l in ["t", "T", "d", "D", "f", "F", "R"] {
            assert_eq!(l.parse::<Style>().unwrap().letter().to_string(), l);
        }
        assert!("x".parse::<Style>().is_err());
    }

    #[test]
    fn arithmetic_and_conversions() {
        let ts = Timestamp::from_unix(BASE);
        assert_eq!(ts.plus_minutes(1).unix(), BASE + 60);
        assert_eq!(ts.plus_hours(-1).unix(), BASE - 3600);
        assert_eq!(ts.plus_days(1).unix(), BASE + 86_400);
        assert_eq!(ts.plus_weeks(1).unix(), BASE + 604_800);
        assert_eq!((ts + Duration::minutes(90)).unix(), BASE + 5400);
        assert_eq!((ts - Duration::milliseconds(1500)).unix(), BASE - 1);
        let st = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(BASE as u64);
        assert_eq!(Timestamp::from(st).unix(), BASE);
        assert_eq!(Timestamp::from_unix(i64::MAX).plus_days(1).unix(), i64::MAX);
        let naive = NaiveDateTime::parse_from_str("2026-11-25 14:30", "%Y-%m-%d %H:%M").unwrap();
        assert_eq!(Timestamp::from(naive).unix(), BASE);
        assert_eq!(
            "2026-11-25 14:30".parse::<Timestamp>().unwrap().unix(),
            BASE
        );
    }

    #[test]
    fn snowflakes() {
        // Example from Discord's API docs: 2016-04-30 11:18:25.796 UTC.
        assert_eq!(
            Timestamp::from_snowflake(175928847299117063).unix(),
            1_462_015_105
        );
        assert_eq!(Timestamp::from_snowflake(0).unix(), 1_420_070_400);
        assert!(Timestamp::from_snowflake(u64::MAX).unix() > 0);
    }

    #[cfg(feature = "tz")]
    #[test]
    fn iana_zone_with_dst() {
        use chrono_tz::Europe::Bratislava;
        // Summer (CEST, UTC+2): 12:00 local = 10:00 UTC.
        assert_eq!(
            Timestamp::parse_in("2026-07-01 12:00", &Bratislava)
                .unwrap()
                .unix(),
            1_782_900_000
        );
        // Clocks jump 02:00 -> 03:00 on 2026-03-29.
        assert!(matches!(
            Timestamp::parse_in("2026-03-29 02:30", &Bratislava),
            Err(FormatError::NonexistentLocalTime(_))
        ));
        assert_eq!(
            Timestamp::parse_in("2026-03-29 03:30", &Bratislava)
                .unwrap()
                .unix(),
            1_774_747_800
        );
        // Clocks go back 03:00 -> 02:00 on 2026-10-25, so 02:30 happens twice.
        assert!(matches!(
            Timestamp::parse_in("2026-10-25 02:30", &Bratislava),
            Err(FormatError::AmbiguousLocalTime(_))
        ));
    }
}
