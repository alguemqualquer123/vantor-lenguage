// Lexicon Standard Library — context.
// Go-parity request contexts (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3).
// Values ride a `[key, value]` pair list (like `maps`); cancellation and
// deadlines are data (checked by `Done`/`Err`), matching Go's observable
// behavior on the single-task scheduler. Import as `import std::context;`.

/// A request context (Go's `context.Context`).
pub struct Context {
    deadline_ms: i64,
    cancelled: bool,
    values: Dynamic,
}

/// Result of `WithCancel`/`WithTimeout`: the derived context. Cancel via
/// `context::Cancel(ctx)` (value semantics — reassign the result).
pub struct CancelResult {
    ctx: Context,
    cancel: Dynamic,
}

/// Lookup result: `value` plus `ok` (Go's `ctx.Value(key)` + nil test).
pub struct ValueResult {
    value: Dynamic,
    ok: bool,
}

/// Returns the empty background context (Go's `context.Background()`).
pub fn Background() -> Context {
    return Context { deadline_ms: 0, cancelled: false, values: [] };
}

/// Returns the TODO placeholder (Go's `context.TODO()`).
pub fn TODO() -> Context {
    return Background();
}

/// Derives a cancellable child plus its cancel function. The cancel
/// function takes no arguments and returns the cancelled context — call
/// it, then keep using `cr.ctx`:
///
/// `let cr = context::WithCancel(ctx); cr.ctx = cr.cancel();`
pub fn WithCancel(parent: Context) -> CancelResult {
    let child = Context { deadline_ms: parent.deadline_ms, cancelled: false, values: parent.values };
    let stopped = Context { deadline_ms: parent.deadline_ms, cancelled: true, values: parent.values };
    let closer = | | stopped;
    return CancelResult { ctx: child, cancel: closer };
}

/// Marks `ctx` cancelled, returning the updated context.
pub fn Cancel(ctx: Context) -> Context {
    return Context { deadline_ms: ctx.deadline_ms, cancelled: true, values: ctx.values };
}

/// Derives a child that expires `timeout_ms` from now (Go's
/// `context.WithTimeout`). Reassign like `WithCancel`.
pub fn WithTimeout(parent: Context, timeout_ms: i64) -> CancelResult {
    let dl = Time::now_unix_ms() + timeout_ms;
    let child = Context { deadline_ms: dl, cancelled: false, values: parent.values };
    let stopped = Context { deadline_ms: dl, cancelled: true, values: parent.values };
    let closer = | | stopped;
    return CancelResult { ctx: child, cancel: closer };
}

/// Derives a child carrying `key -> value` (Go's `context.WithValue`).
pub fn WithValue(parent: Context, key: Dynamic, value: Dynamic) -> Context {
    let vals = parent.values;
    vals.push([key, value]);
    return Context { deadline_ms: parent.deadline_ms, cancelled: parent.cancelled, values: vals };
}

/// Looks up `key` (Go's `ctx.Value(key)`).
pub fn Value(ctx: Context, key: Dynamic) -> ValueResult {
    let i = ctx.values.len() - 1;
    while i >= 0 {
        if ctx.values[i][0] == key {
            return ValueResult { value: ctx.values[i][1], ok: true };
        }
        i = i - 1;
    }
    return ValueResult { value: 0, ok: false };
}

/// Reports whether `ctx` is done (cancelled or past deadline).
pub fn Done(ctx: Context) -> bool {
    if ctx.cancelled {
        return true;
    }
    if ctx.deadline_ms > 0 && Time::now_unix_ms() >= ctx.deadline_ms {
        return true;
    }
    return false;
}

/// Explains why `ctx` is done: "" when live, "Canceled" or
/// "DeadlineExceeded" (Go's `ctx.Err()`).
pub fn Err(ctx: Context) -> String {
    if ctx.cancelled {
        return "Canceled";
    }
    if ctx.deadline_ms > 0 && Time::now_unix_ms() >= ctx.deadline_ms {
        return "DeadlineExceeded";
    }
    return "";
}
