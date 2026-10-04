// Lexicon Standard Library — time.
// Go-parity wall-clock basics (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
// Durations are i64 milliseconds (Go uses nanoseconds; ms keeps every
// value exactly representable in the interpreter). Clock reads delegate
// to the native `Time::*` builtins (single syscall, no interpreter loop).

/// One millisecond in Duration units.
pub const Millisecond = 1;
/// One second in Duration units.
pub const Second = 1000;
/// One minute in Duration units.
pub const Minute = 60000;
/// One hour in Duration units.
pub const Hour = 3600000;

/// An instant: milliseconds since the Unix epoch (Go's `time.Time`,
/// millisecond resolution).
pub struct Time {
    ms: i64,
}

/// Returns the current wall-clock time (Go's `time.Now`).
pub fn Now() -> Time {
    return Time { ms: Time::now_unix_ms() };
}

/// Builds a `Time` from Unix seconds (Go's `time.Unix(sec, 0)`).
pub fn Unix(sec: i64) -> Time {
    return Time { ms: sec * 1000 };
}

/// Builds a `Time` from Unix milliseconds.
pub fn UnixMilli(ms: i64) -> Time {
    return Time { ms: ms };
}

/// Returns the Unix seconds of `t` (Go's `t.Unix()`).
pub fn UnixOf(t: Time) -> i64 {
    return t.ms / 1000;
}

/// Returns the Unix microseconds of `t` (Go's `t.UnixMicro()`).
pub fn UnixMicroOf(t: Time) -> i64 {
    return t.ms * 1000;
}

/// Builds a `Time` from Unix microseconds (Go's `time.UnixMicro`).
pub fn UnixMicro(us: i64) -> Time {
    return Time { ms: us / 1000 };
}

/// Returns the Unix milliseconds of `t` (Go's `t.UnixMilli()`).
pub fn UnixMilliOf(t: Time) -> i64 {
    return t.ms;
}

/// Suspends execution for `ms` milliseconds (Go's `time.Sleep`).
pub fn Sleep(ms: i64) -> void {
    Time::sleep(ms);
}

/// Returns `t + d` where `d` is a Duration in milliseconds.
pub fn Add(t: Time, d: i64) -> Time {
    return Time { ms: t.ms + d };
}

/// Returns the duration `a - b` in milliseconds (Go's `t.Sub`).
pub fn Sub(a: Time, b: Time) -> i64 {
    return a.ms - b.ms;
}

/// Milliseconds elapsed since `t` (Go's `time.Since`).
pub fn Since(t: Time) -> i64 {
    return Now().ms - t.ms;
}

/// Reports whether `a` is before `b`.
pub fn Before(a: Time, b: Time) -> bool {
    let x = a.ms;
    let y = b.ms;
    return x < y;
}

/// Reports whether `a` is after `b`.
pub fn After(a: Time, b: Time) -> bool {
    let x = a.ms;
    let y = b.ms;
    return x > y;
}

/// Reports whether `a` and `b` are the same instant.
pub fn Equal(a: Time, b: Time) -> bool {
    return a.ms == b.ms;
}

/// Formats `t` (UTC) for the common layouts:
/// `"2006-01-02"`, `"15:04:05"`, `"2006-01-02 15:04:05"` and `"RFC3339"`
/// (`2006-01-02T15:04:05Z`). Anything else falls back to the Unix
/// milliseconds (documented subset of Go's reference-layout DSL).
pub fn Format(t: Time, layout: String) -> String {
    let days = time_div(t.ms, 86400000);
    let rem = t.ms - days * 86400000;
    if rem < 0 {
        rem = rem + 86400000;
        days = days - 1;
    }
    let hh = time_div(rem, 3600000);
    let mi = time_div(rem - hh * 3600000, 60000);
    let ss = time_div(rem - hh * 3600000 - mi * 60000, 1000);
    let date = time_date(days);
    let clock = time_two(hh) + ":" + time_two(mi) + ":" + time_two(ss);
    if layout == "2006-01-02" {
        return date;
    }
    if layout == "15:04:05" {
        return clock;
    }
    if layout == "RFC3339" {
        return date + "T" + clock + "Z";
    }
    if layout == "2006-01-02 15:04:05" {
        return date + " " + clock;
    }
    return t.ms.to_string();
}

fn time_div(a: i64, b: i64) -> i64 {
    if a < 0 {
        return 0 - ((0 - a + b - 1) / b);
    }
    return a / b;
}

fn time_two(n: i64) -> String {
    if n < 10 {
        return "0" + n.to_string();
    }
    return n.to_string();
}

// Howard Hinnant's days-to-civil (proleptic Gregorian, UTC).
fn time_date(z: i64) -> String {
    let zz = z + 719468;
    let era = time_div(zz, 146097);
    if zz < 0 {
        era = era - 1;
    }
    let doe = zz - era * 146097;
    let yoe = time_div(doe - time_div(doe, 1460) + time_div(doe, 36524) - time_div(doe, 146096), 365);
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + time_div(yoe, 4) - time_div(yoe, 100));
    let mp = time_div(5 * doy + 2, 153);
    let d = doy - time_div(153 * mp + 2, 5) + 1;
    let m = mp + 3;
    if mp >= 10 {
        m = mp - 9;
        y = y + 1;
    }
    return y.to_string() + "-" + time_two(m) + "-" + time_two(d);
}

/// Parse result: the `time` plus `ok` (Go's `(t, err)` pair).
pub struct ParseResult {
    time: Time,
    ok: bool,
}

/// Parses `"2006-01-02"` (optionally + `" 15:04:05"`) as UTC
/// (Go's `time.Parse` for the two layouts `Format` emits).
pub fn ParseDate(s: String) -> ParseResult {
    let fail = ParseResult { time: Time { ms: 0 }, ok: false };
    if s.len() != 10 && s.len() != 19 {
        return fail;
    }
    let y = time_num(Text::slice(s, 0, 4));
    let mo = time_num(Text::slice(s, 5, 7));
    let d = time_num(Text::slice(s, 8, 10));
    if y < 0 || mo < 1 || mo > 12 || d < 1 || d > 31 {
        return fail;
    }
    if Text::slice(s, 4, 5) != "-" || Text::slice(s, 7, 8) != "-" {
        return fail;
    }
    let hh = 0;
    let mi = 0;
    let ss = 0;
    if s.len() == 19 {
        hh = time_num(Text::slice(s, 11, 13));
        mi = time_num(Text::slice(s, 14, 16));
        ss = time_num(Text::slice(s, 17, 19));
        if hh < 0 || hh > 23 || mi < 0 || mi > 59 || ss < 0 || ss > 59 {
            return fail;
        }
        if Text::slice(s, 10, 11) != " " || Text::slice(s, 13, 14) != ":" || Text::slice(s, 16, 17) != ":" {
            return fail;
        }
    }
    let days = time_days(y, mo, d);
    return ParseResult { time: Time { ms: days * 86400000 + hh * 3600000 + mi * 60000 + ss * 1000 }, ok: true };
}

fn time_num(s: String) -> i64 {
    if s.len() == 0 {
        return -1;
    }
    let v = 0;
    let i = 0;
    while i < s.len() {
        let d = Text::index_of("0123456789", Text::slice(s, i, i + 1));
        if d < 0 {
            return -1;
        }
        v = v * 10 + d;
        i = i + 1;
    }
    return v;
}

// Inverse of time_date: civil date to days since the Unix epoch.
fn time_days(y: i64, m: i64, d: i64) -> i64 {
    let yy = y;
    let mm = m;
    if mm <= 2 {
        yy = yy - 1;
        mm = mm + 12;
    }
    let era = time_div(yy, 400);
    let yoe = yy - era * 400;
    let mp = mm - 3;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    return era * 146097 + doe - 719468;
}
