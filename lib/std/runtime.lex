// Lexicon Standard Library — runtime.
// Go-parity runtime facts (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 8).
// Every value below is real: OS/arch come from the native `Sys::*`
// builtins, the version from the toolchain. Import as
// `import std::runtime;`.

/// Compiler version reported by the toolchain (Go's `runtime.Version`).
/// Delegates to the native build version, so release auto-bumps are
/// always reflected (never a stale literal).
pub fn Version() -> String {
    return "lex" + Sys::lex_version();
}

/// Operating system: `windows`, `linux`, `macos`, … (Go's `GOOS`,
/// via the native `Sys::goos`).
pub fn GOOS() -> String {
    return Sys::goos();
}

/// Architecture: `x86_64`, `aarch64`, … (Go's `GOARCH`).
pub fn GOARCH() -> String {
    return Sys::arch();
}

/// Logical CPUs usable (Go's `runtime.NumCPU`, native).
pub fn NumCPU() -> i64 {
    return Sys::ncpu();
}

/// Goroutines live in the interpreter (Go's `runtime.NumGoroutine`):
/// the tree-walker runs one task — cooperative tasks arrive in Fase 8.
pub fn NumGoroutine() -> i64 {
    return 1;
}

/// Sets the maximum threads (Go's `runtime.GOMAXPROCS`): recorded and
/// echoed back; the single-task scheduler honours it trivially.
pub fn GOMAXPROCS(n: i64) -> i64 {
    gomax = n;
    return n;
}

let gomax = 8;

/// Yields the processor (Go's `runtime.Gosched`): no-op on the
/// single-task scheduler, kept for porting.
pub fn Gosched() -> void {
}

/// Runs the garbage collector cycle now (Go's `runtime.GC`): values are
/// reference-counted by scope exit, so this only documents the point.
pub fn GC() -> void {
}
