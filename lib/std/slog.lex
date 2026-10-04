// Lexicon Standard Library — slog.
// Go-parity structured logging (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3).
// Attributes ride `[key, value]` pair lists; the text handler writes one
// line per record to stdout. Import as `import std::slog;`.

/// Log level (Go's `slog.Level`: Debug -4, Info 0, Warn 4, Error 8).
pub struct Level {
    n: i64,
}

/// A logger: minimum level plus bound attributes (Go's `slog.Logger`).
pub struct Logger {
    level: i64,
    attrs: Dynamic,
}

pub const LevelDebug = -4;
pub const LevelInfo = 0;
pub const LevelWarn = 4;
pub const LevelError = 8;

let slog_level = 0;

/// Sets the default level (0 = Info).
pub fn SetLevel(n: i64) -> void {
    slog_level = n;
}

/// Builds a logger at `level` (use the `Level*` constants).
pub fn New(level: i64) -> Logger {
    return Logger { level: level, attrs: [] };
}

/// Returns a logger with extra attributes bound (Go's `With`).
pub fn With(l: Logger, attrs: Dynamic) -> Logger {
    let merged = l.attrs;
    let i = 0;
    while i < attrs.len() {
        merged.push(attrs[i]);
        i = i + 1;
    }
    return Logger { level: l.level, attrs: merged };
}

fn lvl_name(n: i64) -> String {
    if n <= -4 {
        return "DEBUG";
    }
    if n >= 8 {
        return "ERROR";
    }
    if n >= 4 {
        return "WARN";
    }
    return "INFO";
}

fn emit(l: Logger, n: i64, msg: String, attrs: Dynamic) -> void {
    if n < l.level {
        return;
    }
    let line = lvl_name(n) + " " + msg;
    let all = l.attrs;
    let i = 0;
    while i < attrs.len() {
        all.push(attrs[i]);
        i = i + 1;
    }
    let j = 0;
    while j < all.len() {
        line = line + " " + all[j][0].to_string() + "=" + all[j][1].to_string();
        j = j + 1;
    }
    Console::log(line);
}

/// Logs at Debug (Go's `logger.Debug`).
pub fn Debug(l: Logger, msg: String, attrs: Dynamic) -> void {
    emit(l, -4, msg, attrs);
}

/// Logs at Info.
pub fn Info(l: Logger, msg: String, attrs: Dynamic) -> void {
    emit(l, 0, msg, attrs);
}

/// Logs at Warn.
pub fn Warn(l: Logger, msg: String, attrs: Dynamic) -> void {
    emit(l, 4, msg, attrs);
}

/// Logs at Error.
pub fn Error(l: Logger, msg: String, attrs: Dynamic) -> void {
    emit(l, 8, msg, attrs);
}

/// Default-logger Debug (Go's `slog.Debug`).
pub fn Dbg(msg: String) -> void {
    emit(New(slog_level), -4, msg, []);
}

/// Default-logger Info (Go's `slog.Info`).
pub fn Inf(msg: String) -> void {
    emit(New(slog_level), 0, msg, []);
}
