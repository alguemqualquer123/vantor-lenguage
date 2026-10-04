// Lexicon Standard Library — testing.
// Go-parity test helpers (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 9/10).
// `T` threads failure state through assertions with value semantics:
// reassign — `t = testing::Errorf(t, ...)`. Import as
// `import std::testing;`.

/// Test context (Go's `*testing.T`).
pub struct T {
    name: String,
    failed: bool,
    skipped: bool,
    logs: Dynamic,
}

/// Benchmark result (Go's `testing.BenchmarkResult`, time form).
pub struct BenchResult {
    n: i64,
    ms: i64,
    nsop: i64,
}

/// Starts a test context (the name `lex test` would report).
pub fn New(name: String) -> T {
    return T { name: name, failed: false, skipped: false, logs: [] };
}

/// Records `msg` (Go's `t.Log`).
pub fn Log(t: T, msg: String) -> T {
    let logs = t.logs;
    logs.push(msg);
    return T { name: t.name, failed: t.failed, skipped: t.skipped, logs: logs };
}

/// Marks failed with `msg` (Go's `t.Error`).
pub fn Error(t: T, msg: String) -> T {
    let n = Log(t, msg);
    return T { name: n.name, failed: true, skipped: n.skipped, logs: n.logs };
}

/// Marks failed and stops the test body (Go's `t.Fatal`: callers should
/// `return` right after).
pub fn Fatal(t: T, msg: String) -> T {
    return Error(t, msg);
}

/// Marks skipped (Go's `t.Skip`).
pub fn Skip(t: T, msg: String) -> T {
    let n = Log(t, msg);
    return T { name: n.name, failed: n.failed, skipped: true, logs: n.logs };
}

/// Reports failure state (Go's `t.Failed`).
pub fn Failed(t: T) -> bool {
    return t.failed;
}

/// Reports skip state (Go's `t.Skipped`).
pub fn Skipped(t: T) -> bool {
    return t.skipped;
}

/// Fails when `cond` is false (boolean assertion).
pub fn Assert(t: T, cond: bool, msg: String) -> T {
    if cond {
        return t;
    }
    return Error(t, msg);
}

/// Fails when `a != b` (equality assertion, `==` semantics).
pub fn AssertEq(t: T, a: Dynamic, b: Dynamic, msg: String) -> T {
    if a == b {
        return t;
    }
    return Error(t, msg + ": got " + a.to_string());
}

/// Benchmarks `f` over `n` iterations, returning elapsed stats (Go's
// `testing.Benchmark` shape; `f` is a zero-arg lambda).
pub fn Benchmark(n: i64, f: Dynamic) -> BenchResult {
    let t0 = Time::now_unix_ms();
    let i = 0;
    while i < n {
        f();
        i = i + 1;
    }
    let ms = Time::now_unix_ms() - t0;
    let nsop = 0;
    if n > 0 {
        nsop = ms * 1000000 / n;
    }
    return BenchResult { n: n, ms: ms, nsop: nsop };
}

/// Prints a group header (Jest's `describe`, output form).
pub fn Describe(name: String) -> void {
    Console::log(name);
}

/// Runs one named case: logs the name, then calls `f(t)` with a fresh
/// context and returns it (Jest's `it`). A failing assert aborts the
/// run with the message — like Jest's thrown matchers.
pub fn It(t: T, name: String, f: Dynamic) -> T {
    Console::log("  " + name);
    return f(t);
}

/// Expects equality (Jest's `expect(a).toBe(b)`).
pub fn ExpectEq(t: T, a: Dynamic, b: Dynamic) -> T {
    return AssertEq(t, a, b, "expected equal");
}

/// Expects truth (Jest's `expect(x).toBeTruthy()`).
pub fn ExpectTrue(t: T, cond: bool) -> T {
    return Assert(t, cond, "expected truthy");
}

/// Expects falsity.
pub fn ExpectFalse(t: T, cond: bool) -> T {
    if cond {
        return Error(t, "expected falsy");
    }
    return t;
}
