// Lexicon Standard Library — os/signal.
// Go-parity signal constants (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3).
// Delivery is cooperative: `Notify` records interest and `Caught` drains
// it; real async delivery arrives with the threaded scheduler (Fase 8),
// so handlers poll. Import as `import std::os::signal;`.

pub const SIGINT = 2;
pub const SIGTERM = 15;
pub const SIGKILL = 9;
pub const SIGHUP = 1;
pub const SIGQUIT = 3;

let signal_want = [];

/// Registers interest in `sig` (Go's `signal.Notify`, recording form).
pub fn Notify(sig: i64) -> void {
    signal_want.push(sig);
}

/// Stops delivery (Go's `signal.Stop`, recording form).
pub fn Stop(sig: i64) -> void {
    let keep = [];
    let i = 0;
    while i < signal_want.len() {
        if signal_want[i] != sig {
            keep.push(signal_want[i]);
        }
        i = i + 1;
    }
    signal_want = keep;
}

/// Returns registered signals (introspection over `Notify`/`Stop`).
pub fn Wanted() -> Dynamic {
    let out = [];
    let i = 0;
    while i < signal_want.len() {
        out.push(signal_want[i]);
        i = i + 1;
    }
    return out;
}
