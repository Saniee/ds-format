# ds-format

Helpers for Discord's text formatting: mentions, timestamps and markdown. Works with any Discord library (including [ds-webhook](https://github.com/Saniee/ds-webhook)), and everything returns plain `String`s.

```toml
[dependencies]
ds-format = { git = "https://github.com/Saniee/ds-format" }
# or, to read dates in zones like Europe/Bratislava:
ds-format = { git = "https://github.com/Saniee/ds-format", features = ["tz"] }
```

One import brings in everything:

```rust
use ds_format::prelude::*;

let ends = Timestamp::parse("2026-11-25 18:00")?;
let msg = format!("{} ends {}", mention::role(1234), ends.relative().bold());
```

## Timestamps

Discord shows `<t:…>` timestamps in each viewer's own time zone and language.

```rust
use ds_format::{Style, Timestamp};

let ts = Timestamp::parse("2026-11-25 14:30")?;   // no offset → read as UTC
ts.relative();                 // "<t:1795617000:R>"  → "in 2 hours"
ts.format(Style::LongDate);    // "<t:1795617000:D>"  → "25 November 2026"
Timestamp::now().plus_hours(2).relative();
format!("ends {}", ts.styled(Style::Relative)); // same, without an extra String
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

**Other sources:**

```rust
Timestamp::from_snowflake(message_id);   // when a user/message/channel was created
Timestamp::from(datetime);               // any chrono DateTime
Timestamp::from(naive);                  // NaiveDateTime, read as UTC
Timestamp::from(SystemTime::now());
Timestamp::from_unix(secs);
ts + chrono::Duration::minutes(90);      // also plus_minutes/hours/days/weeks
```

## Mentions

IDs are plain `u64`s; integer literals need no suffix.

```rust
use ds_format::mention::{self, Navigation};

mention::user(80351110224678912);       // <@80351110224678912>
mention::role(42);                      // <@&42>
mention::channel(7);                    // <#7>
mention::slash_command("config set", 9); // </config set:9>
mention::emoji("blob", 5, true);        // <a:blob:5>
mention::everyone();                    // @everyone
mention::navigation(Navigation::Customize); // <id:customize>
mention::message_link(1, 2, 3);         // https://discord.com/channels/1/2/3
```

ID types from Discord libraries convert with `.get()` (or `u64::from(id)`). `Mention` values also parse back from text: `"<@!42>".parse::<Mention>()`.

Whether a mention actually notifies someone is controlled by the message's `allowed_mentions`, not by the text.

## Markdown

```rust
use ds_format::markdown::*;

bold("x"); italic("x"); bold_italic("x"); underline("x"); strike("x"); spoiler("x");
code("x"); code_block("rs", "let x = 1;"); quote("a\nb"); block_quote("a\nb");
header(2, "Title"); subtext("small print");
list(["a", "b"]); numbered_list(["a", "b"]);
link("https://…");                      // <https://…>, no embed preview
masked_link("docs", "https://…");       // [docs](https://…)
masked_link_no_embed("docs", "https://…");

escape(user_input); // so "*text*" or "<@1>" from a user shows up literally
```

The wrappers accept anything printable, so helpers combine without `&`: `bold(ts.relative())`, `italic(mention::user(1))`, `bold(42)`.

Or chain them as methods with the `Format` trait:

```rust
use ds_format::Format;

user_input.escaped().bold();   // **…**
ts.relative().italic();        // *<t:…:R>*
```

## Limits

```rust
use ds_format::limits::{self, split_message, truncate};

truncate(title, limits::EMBED_TITLE);       // cut with "…" if over 256 chars
for part in split_message(&long_text, limits::MESSAGE) {
    // each part ≤ 2000 chars; code blocks are closed and reopened across parts
}
```

Constants: `MESSAGE`, `EMBED_TITLE`, `EMBED_DESCRIPTION`, `FIELD_NAME`, `FIELD_VALUE`, `FOOTER`, `AUTHOR_NAME`, `EMBED_TOTAL`.
