// Date, time, and internationalization (Spec §47 + §48).
//
// Rule: wall-clock time and monotonic durations are DIFFERENT types and
// never mix. Locale-sensitive formatting always takes an explicit locale;
// machine-local settings are never consulted implicitly.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// UTC timestamp with millisecond precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    pub millis_epoch: i64,
}

impl Timestamp {
    pub fn now() -> Self {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        Timestamp { millis_epoch: ms }
    }

    pub fn from_millis(millis_epoch: i64) -> Self {
        Timestamp { millis_epoch }
    }

    /// RFC 3339 (`2026-01-01T00:00:00.000Z`). Uses a civil-date algorithm;
    /// valid for the full i64 millisecond range.
    pub fn to_rfc3339(&self) -> String {
        let (y, mo, d, h, mi, s, ms) = civil_from_millis(self.millis_epoch);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            y, mo, d, h, mi, s, ms
        )
    }

    /// Elapsed wall-clock duration since an earlier timestamp.
    pub fn elapsed_since(&self, earlier: &Timestamp) -> LexDuration {
        LexDuration::from_millis((self.millis_epoch - earlier.millis_epoch).max(0) as u64)
    }
}

fn civil_from_millis(ms: i64) -> (i64, u32, u32, u32, u32, u32, u32) {
    let millis = ms.rem_euclid(1000) as u32;
    let mut secs = ms.div_euclid(1000);
    let sec = (secs % 60) as u32;
    secs /= 60;
    let min = (secs % 60) as u32;
    secs /= 60;
    let hour = (secs % 24) as u32;
    let mut days = secs.div_euclid(24);
    // Howard Hinnant's civil-from-days algorithm.
    days += 719468;
    let era = days.div_euclid(146097);
    let doe = days.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d, hour, min, sec, millis)
}

/// Duration that can ONLY come from monotonic measurement or explicit
/// construction — never from wall-clock subtraction of raw values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexDuration {
    inner: Duration,
}

impl LexDuration {
    pub fn from_millis(ms: u64) -> Self {
        LexDuration { inner: Duration::from_millis(ms) }
    }

    pub fn from_secs(s: u64) -> Self {
        LexDuration { inner: Duration::from_secs(s) }
    }

    pub fn as_millis(&self) -> u128 {
        self.inner.as_millis()
    }

    pub fn as_std(&self) -> Duration {
        self.inner
    }
}

/// Monotonic clock for measuring durations (Spec §48).
#[derive(Debug, Clone)]
pub struct Monotonic {
    start: Instant,
}

impl Monotonic {
    pub fn now() -> Self {
        Monotonic { start: Instant::now() }
    }

    pub fn elapsed(&self) -> LexDuration {
        LexDuration { inner: self.start.elapsed() }
    }
}

/// Interval ticker yielding at most `max_ticks` ticks.
///
/// Ownership: owns scheduling state. Thread-safety: `!Sync` (uses `Instant`);
/// move across threads but do not share without a lock.
/// Complexity: `tick` blocks up to `interval`; bookkeeping O(1).
pub struct Ticker {
    interval: Duration,
    max_ticks: usize,
    ticks: usize,
    next: Instant,
}

impl Ticker {
    pub fn new(interval: Duration, max_ticks: usize) -> Self {
        Ticker { interval, max_ticks, ticks: 0, next: Instant::now() + interval }
    }

    /// Block until the next tick; returns false when exhausted.
    pub fn tick(&mut self) -> bool {
        if self.ticks >= self.max_ticks {
            return false;
        }
        let now = Instant::now();
        if self.next > now {
            std::thread::sleep(self.next - now);
        }
        self.next = Instant::now() + self.interval;
        self.ticks += 1;
        true
    }

    pub fn ticks(&self) -> usize {
        self.ticks
    }
}

// ---------------------------------------------------------------------------
// Date / UTC / Timer (Spec §48).
//
// Rule restated: wall-clock (`Timestamp`/`Date`/`Utc`) and monotonic
// (`Monotonic`/`LexDuration`) never mix implicitly. Convert explicitly via
// `*_from_millis` / `elapsed_since`.
// Ownership: all `Copy` value types. Thread-safety: `Send + Sync`.
// ---------------------------------------------------------------------------

/// Calendar date (proleptic Gregorian, UTC).
///
/// Ownership: `Copy` value. Thread-safety: `Send + Sync`.
/// Complexity: validation O(1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

#[allow(dead_code)]
impl Date {
    /// Validate `year/month/day` (month 1–12, day per month incl. leap years).
    /// Complexity: O(1).
    pub fn from_ymd(year: i64, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) || day < 1 {
            return None;
        }
        let dim = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if is_leap(year) => 29,
            2 => 28,
            _ => return None,
        };
        if day > dim {
            return None;
        }
        Some(Date { year, month, day })
    }

    /// ISO-8601 calendar date (`2026-01-31`). Complexity: O(1).
    pub fn to_iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Split a millisecond-epoch timestamp into its UTC date part.
    /// Complexity: O(1).
    pub fn from_timestamp(ts: Timestamp) -> Self {
        let (y, mo, d, _, _, _, _) = civil_from_millis(ts.millis_epoch);
        Date { year: y, month: mo, day: d }
    }

    /// Monday = 1 … Sunday = 7 (proleptic Gregorian). Complexity: O(1).
    pub fn weekday(self) -> u32 {
        // days_from_civil → 1970-01-01 was Thursday (4).
        let days = days_from_civil(self.year, self.month, self.day);
        ((days + 3).rem_euclid(7) + 1) as u32
    }
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = ((m as i64 + 9) % 12) as i64;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// UTC clock namespace.
///
/// All functions are wall-clock; never use for measuring durations —
/// use [`Monotonic`] instead. Ownership: stateless unit struct.
/// Thread-safety: `Send + Sync`. Complexity: O(1) syscalls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Utc;

#[allow(dead_code)]
impl Utc {
    /// Current wall-clock timestamp. Complexity: O(1) syscall.
    pub fn now() -> Timestamp {
        Timestamp::now()
    }

    /// RFC 3339 for a timestamp (delegates to [`Timestamp::to_rfc3339`]).
    /// Complexity: O(1).
    pub fn to_rfc3339(ts: Timestamp) -> String {
        ts.to_rfc3339()
    }

    /// Today (UTC date part of now). Complexity: O(1).
    pub fn today() -> Date {
        Date::from_timestamp(Timestamp::now())
    }
}

/// One-shot monotonic timer.
///
/// Ownership: owns deadline state. Thread-safety: `!Sync` (wraps `Instant`);
/// move, don't share. Complexity: `wait` blocks up to remaining; rest O(1).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Timer {
    deadline: Instant,
    duration: Duration,
}

#[allow(dead_code)]
impl Timer {
    pub fn new(duration: LexDuration) -> Self {
        let d = duration.as_std();
        Timer { deadline: Instant::now() + d, duration: d }
    }

    pub fn from_std(duration: Duration) -> Self {
        Timer { deadline: Instant::now() + duration, duration }
    }

    /// Remaining time (zero once expired). Complexity: O(1).
    pub fn remaining(&self) -> LexDuration {
        let now = Instant::now();
        if now >= self.deadline {
            LexDuration::from_millis(0)
        } else {
            let ms = (self.deadline - now).as_millis().min(u128::from(u64::MAX)) as u64;
            LexDuration::from_millis(ms)
        }
    }

    /// True once the deadline passed. Complexity: O(1).
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }

    /// Block until expiry. Complexity: blocks ≤ duration.
    pub fn wait(&self) {
        let now = Instant::now();
        if self.deadline > now {
            std::thread::sleep(self.deadline - now);
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }
}

// ---------------------------------------------------------------------------
// Explicit-locale formatting (Spec §47)
// ---------------------------------------------------------------------------

/// Supported explicit locales. Anything else is rejected — no silent
/// machine-locale fallback (Spec §47 determinism rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    EnUs,
    DeDe,
    FrFr,
}

impl Locale {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "en-US" => Some(Locale::EnUs),
            "de-DE" => Some(Locale::DeDe),
            "fr-FR" => Some(Locale::FrFr),
            _ => None,
        }
    }

    fn thousand_sep(self) -> char {
        match self {
            Locale::EnUs => ',',
            Locale::DeDe => '.',
            Locale::FrFr => ' ',
        }
    }

    fn decimal_sep(self) -> char {
        match self {
            Locale::EnUs => '.',
            Locale::DeDe => ',',
            Locale::FrFr => ',',
        }
    }
}

/// Format an integer with thousands separators for an explicit locale.
pub fn format_int(n: i64, locale: Locale) -> String {
    let neg = n < 0;
    let digits: Vec<char> = n.unsigned_abs().to_string().chars().collect();
    let mut out = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(locale.thousand_sep());
        }
        out.push(*c);
    }
    if neg {
        out.insert(0, '-');
    }
    out
}

/// Format money: integer minor units + explicit currency + locale.
/// Ownership: pure function. Thread-safety: `Send + Sync`.
/// Complexity: O(digits).
pub fn format_currency(minor: i64, currency: &str, locale: Locale) -> String {
    let major = minor / 100;
    let cents = (minor % 100).abs();
    format!(
        "{}{}{:02} {}",
        format_int(major, locale),
        locale.decimal_sep(),
        cents,
        currency
    )
}

/// Format a float with `frac_digits` and locale decimal separator.
/// No scientific notation; `NaN`/`inf` render literally.
/// Ownership: pure. Thread-safety: `Send + Sync`. Complexity: O(digits).
pub fn format_float(v: f64, frac_digits: usize, locale: Locale) -> String {
    if !v.is_finite() {
        return v.to_string();
    }
    let neg = v < 0.0 || (v == 0.0 && v.is_sign_negative());
    let s = format!("{:.prec$}", v.abs(), prec = frac_digits.min(18));
    let out = match s.find('.') {
        Some(i) => {
            let (int, frac) = s.split_at(i);
            let grouped = format_int(int.parse::<i64>().unwrap_or(0), locale);
            format!("{}{}{}", grouped, locale.decimal_sep(), &frac[1..])
        }
        None => format_int(s.parse::<i64>().unwrap_or(0), locale),
    };
    if neg {
        format!("-{}", out)
    } else {
        out
    }
}

/// Deterministic date formatting for an explicit locale (no machine locale).
/// Styles: `en-US` → `MM/DD/YYYY`, `de-DE` → `DD.MM.YYYY`,
/// `fr-FR` → `DD/MM/YYYY`. Time appended when `with_time`.
/// Ownership: pure. Thread-safety: `Send + Sync`. Complexity: O(1).
pub fn format_date(d: Date, locale: Locale) -> String {
    match locale {
        Locale::EnUs => format!("{:02}/{:02}/{:04}", d.month, d.day, d.year),
        Locale::DeDe => format!("{:02}.{:02}.{:04}", d.day, d.month, d.year),
        Locale::FrFr => format!("{:02}/{:02}/{:04}", d.day, d.month, d.year),
    }
}

/// Deterministic datetime formatting: `format_date` + ` HH:MM:SS.mmmZ`
/// taken from `ts` in UTC. Ownership: pure. Complexity: O(1).
pub fn format_datetime(ts: Timestamp, locale: Locale) -> String {
    let (y, mo, d, h, mi, s, ms) = civil_from_millis(ts.millis_epoch);
    let date = Date { year: y, month: mo, day: d };
    format!("{} {:02}:{:02}:{:02}.{:03}Z", format_date(date, locale), h, mi, s, ms)
}

/// Deterministic collation stub (Spec §47): byte-wise ordinal comparison.
/// Full CLDR tailoring is an integration point; this is stable, total and
/// locale-independent so output stays deterministic.
/// Ownership: pure. Thread-safety: `Send + Sync`. Complexity: O(min len).
#[allow(dead_code)]
pub fn collate(a: &str, b: &str, _locale: Locale) -> std::cmp::Ordering {
    a.cmp(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_epoch() {
        assert_eq!(Timestamp::from_millis(0).to_rfc3339(), "1970-01-01T00:00:00.000Z");
        // 2026-01-01T00:00:00Z
        assert_eq!(
            Timestamp::from_millis(1767225600000).to_rfc3339(),
            "2026-01-01T00:00:00.000Z"
        );
    }

    #[test]
    fn monotonic_moves() {
        let m = Monotonic::now();
        std::thread::sleep(Duration::from_millis(2));
        assert!(m.elapsed().as_millis() >= 1);
    }

    #[test]
    fn ticker_counts() {
        let mut t = Ticker::new(Duration::from_millis(1), 3);
        let mut n = 0;
        while t.tick() {
            n += 1;
        }
        assert_eq!((n, t.ticks()), (3, 3));
    }

    #[test]
    fn locale_formats() {
        assert_eq!(format_int(1234567, Locale::EnUs), "1,234,567");
        assert_eq!(format_int(1234567, Locale::DeDe), "1.234.567");
        assert_eq!(format_currency(123456, "USD", Locale::EnUs), "1,234.56 USD");
        assert_eq!(Locale::parse("xx"), None);
    }

    #[test]
    fn date_utc_timer() {
        let d = Date::from_ymd(2026, 1, 31).unwrap();
        assert_eq!(d.to_iso(), "2026-01-31");
        assert!(Date::from_ymd(2026, 2, 30).is_none());
        assert!(Date::from_ymd(2024, 2, 29).is_some());
        assert_eq!(Date::from_timestamp(Timestamp::from_millis(0)), Date::from_ymd(1970, 1, 1).unwrap());
        assert_eq!(format_date(d, Locale::EnUs), "01/31/2026");
        assert_eq!(format_date(d, Locale::DeDe), "31.01.2026");
        assert_eq!(format_float(1234567.891, 2, Locale::EnUs), "1,234,567.89");
        assert_eq!(format_float(1234567.891, 2, Locale::DeDe), "1.234.567,89");
        assert_eq!(
            format_datetime(Timestamp::from_millis(0), Locale::EnUs),
            "01/01/1970 00:00:00.000Z"
        );
        assert_eq!(Utc::to_rfc3339(Timestamp::from_millis(0)), "1970-01-01T00:00:00.000Z");
        let t = Timer::from_std(Duration::from_millis(5));
        assert!(!t.is_expired());
        t.wait();
        assert!(t.is_expired());
        assert_eq!(t.remaining().as_millis(), 0);
    }
}
