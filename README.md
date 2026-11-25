# ds-format

Helpers for Discord's text formatting: mentions, timestamps and markdown. Works with any Discord library (including [ds-webhook](https://github.com/Saniee/ds-webhook)), and everything returns plain `String`s.

```toml
[dependencies]
ds-format = { git = "https://github.com/Saniee/ds-format" }
# or, to read dates in zones like Europe/Bratislava:
ds-format = { git = "https://github.com/Saniee/ds-format", features = ["tz"] }
```

## Timestamps

Discord shows `<t:…>` timestamps in each viewer's own time zone and language.

```rust
use ds_format::{Style, Timestamp};

let ts = Timestamp::parse("2026-11-25 14:30")?;   // no offset → read as UTC
ts.relative();                 // "<t:1795617000:R>"  → "in 2 hours"
ts.format(Style::LongDate);    // "<t:1795617000:D>"  → "25 November 2026"
Timestamp::now().plus_hours(2).relative();
```

| Style | Letter | Looks like |
| --- | --- | --- |
| `ShortTime` | `t` | 14:30 |
| `LongTime` | `T` | 14:30:00 |
| `ShortDate` | `d` | 25/11/2026 |
| `LongDate` | `D` | 25 November 2026 |
| `ShortDateTime` (default) | `f` | 25 November 2026 14:30 |
| `LongDateTime` | `F` | Wednesday, 25 November 2026 14:30 |
| `Relative` | `R` | in 2 hours |

**Accepted date text:** `2026-11-25`, `2026-11-25 14:30`, `2026-11-25 14:30:15`, `T` instead of the space, optionally ending in `Z`, `+01:00` or `+0100`.

**Time zones:** a date without an offset is read as UTC. To read it in another zone, use `parse_in`. An offset written in the text always wins.

```rust
use ds_format::chrono::FixedOffset;
use ds_format::chrono_tz::Europe::Bratislava; // needs the `tz` feature

Timestamp::parse_in("2026-11-25 14:30", &FixedOffset::east_opt(3600).unwrap())?;
Timestamp::parse_in("2026-07-01 12:00", &Bratislava)?; // knows about summer time
```

With an IANA zone, times that are skipped or repeated when the clocks change return `NonexistentLocalTime` or `AmbiguousLocalTime` instead of guessing.

You can also convert from chrono values: `Timestamp::from(datetime)`, `Timestamp::from(naive)` (read as UTC), or `Timestamp::from_unix(secs)`.

## Mentions

```rust
use ds_format::mention;

mention::user(80351110224678912u64);    // <@80351110224678912>
mention::role(42u64);                   // <@&42>
mention::channel(7u64);                 // <#7>
mention::slash_command("config set", 9u64); // </config set:9>
mention::emoji("blob", 5u64, true);     // <a:blob:5>
mention::everyone();                    // @everyone
```

Whether a mention actually notifies someone is controlled by the message's `allowed_mentions`, not by the text.

## Markdown

```rust
use ds_format::markdown::*;

bold("x"); italic("x"); underline("x"); strike("x"); spoiler("x");
code("x"); code_block("rs", "let x = 1;"); quote("a\nb");
header(2, "Title"); subtext("small print"); masked_link("docs", "https://…");

escape(user_input); // so "*text*" from a user shows up literally
```
