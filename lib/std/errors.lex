// Lexicon Standard Library — errors.
// Go-parity error values (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
//
// Go's `error` interface becomes a struct value; `errors.Is` compares
// message identity, matching Go's semantics for values created by New.

/// A Lex error value: the message a Go `error` would carry, plus an
/// optional cause (single error or list, for `Unwrap`/`Join`).
pub struct Error {
    msg: String,
    cause: Dynamic,
}

/// Returns a new error value with the given message (Go's `errors.New`).
pub fn New(msg: String) -> Error {
    return Error { msg: msg, cause: 0 };
}

/// Reports whether `a` equals `b` (Go's `errors.Is`): message identity,
/// unwrapping single causes and `Join` lists.
pub fn Is(a: Error, b: Error) -> bool {
    if a.msg == b.msg {
        return true;
    }
    let c = a.cause;
    if inspect(c) == "null" || c == 0 {
        return false;
    }
    if Text::slice(inspect(c), 0, 1) == "[" {
        let i = 0;
        while i < c.len() {
            if Is(c[i], b) {
                return true;
            }
            i = i + 1;
        }
        return false;
    }
    return Is(c, b);
}

/// Returns the message carried by `e` (Go's `error.Error()`).
pub fn Message(e: Error) -> String {
    return e.msg;
}

/// Formats as the bare message — Go prints errors via their message.
pub fn ErrorString(e: Error) -> String {
    return e.msg;
}

/// Wraps `inner` with `msg` (Go's `fmt.Errorf("…: %w", …)` shape).
pub fn Wrap(msg: String, inner: Dynamic) -> Error {
    return Error { msg: msg, cause: inner };
}

/// Returns the cause (Go's `errors.Unwrap`; 0 when absent).
pub fn Unwrap(e: Error) -> Dynamic {
    return e.cause;
}

/// Joins errors with newlines (Go's `errors.Join`; nils skipped).
pub fn Join(errs: Dynamic) -> Error {
    let msg = "";
    let causes = [];
    let i = 0;
    while i < errs.len() {
        let e = errs[i];
        if inspect(e) != "null" {
            if causes.len() > 0 {
                msg = msg + "\n";
            }
            msg = msg + e.msg;
            causes.push(e);
        }
        i = i + 1;
    }
    return Error { msg: msg, cause: causes };
}
