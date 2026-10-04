// Lexicon Standard Library — bufio.
// Go-parity buffered I/O over `std::io` values
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1). Same value-semantics rule
// as `io`: advancing operations return the updated value — reassign it:
// `let rr = bufio::ReadString(r, "\n"); r = rr.reader;`.

import std::io;

/// A buffered reader: an `io.Reader` plus a one-line lookahead.
pub struct BufReader {
    inner: Dynamic,
    buf: String,
}

/// Result of a delimited read: updated `reader`, the `text` (delimiter
/// included, like Go) and `eof` when the stream is exhausted.
pub struct ReadStringResult {
    reader: BufReader,
    text: String,
    eof: bool,
}

/// A line/word scanner over a string (Go's `bufio.Scanner`).
pub struct Scanner {
    lines: Dynamic,
    idx: i64,
}

/// Wraps an `io.Reader` with a buffer (Go's `bufio.NewReader`).
pub fn NewReader(r: Dynamic) -> BufReader {
    return BufReader { inner: r, buf: "" };
}

/// Reads up to and including `delim` (Go's `ReadString`). The final
/// fragment arrives with `eof == true`.
pub fn ReadString(r: BufReader, delim: String) -> ReadStringResult {
    let data = "";
    if r.buf != "" {
        data = r.buf;
    } else {
        data = r.inner.data;
    }
    let start = 0;
    if r.buf == "" {
        start = r.inner.pos;
    }
    let i = Text::index_of(Text::slice(data, start, data.len()), delim);
    if i >= 0 {
        let end = start + i + delim.len();
        let text = Text::slice(data, start, end);
        let rest = Text::slice(data, end, data.len());
        let next = BufReader { inner: r.inner, buf: rest };
        return ReadStringResult { reader: next, text: text, eof: false };
    }
    let text = Text::slice(data, start, data.len());
    let next = BufReader { inner: r.inner, buf: "" };
    return ReadStringResult { reader: next, text: text, eof: true };
}

/// Builds a line scanner: splits on `\n`, dropping one trailing `\r`
/// per line (Go's `ScanLines`).
pub fn NewScanner(s: String) -> Scanner {
    return Scanner { lines: s.split("\n"), idx: 0 };
}

/// Builds a word scanner (Go's `ScanWords`).
pub fn NewWordScanner(s: String) -> Scanner {
    let words = [];
    let cur = "";
    let i = 0;
    let n = s.len();
    while i < n {
        let c = Text::slice(s, i, i + 1);
        if c == " " || c == "\t" || c == "\n" || c == "\r" {
            if cur != "" {
                words.push(cur);
                cur = "";
            }
        } else {
            cur = cur + c;
        }
        i = i + 1;
    }
    if cur != "" {
        words.push(cur);
    }
    return Scanner { lines: words, idx: 0 };
}

/// Advances to the next token; `false` when exhausted (Go's `Scan`).
/// Value semantics: reassign — `let st = bufio::Scan(sc); sc = st.scanner;`.
pub fn Scan(s: Scanner) -> Dynamic {
    let pos = s.idx;
    let total = s.lines.len();
    if pos < total {
        return [true, s.lines[s.idx]];
    }
    return [false, ""];
}

/// Advances the scanner, returning the updated scanner plus the token.
/// Prefer this over `Scan` when consuming in a loop.
pub fn Next(s: Scanner) -> ScannerResult {
    let pos = s.idx;
    let total = s.lines.len();
    if pos < total {
        let tok = s.lines[s.idx];
        if tok.len() > 0 && Text::slice(tok, tok.len() - 1, tok.len()) == "\r" {
            tok = Text::slice(tok, 0, tok.len() - 1);
        }
        let next = Scanner { lines: s.lines, idx: s.idx + 1 };
        return ScannerResult { scanner: next, text: tok, ok: true };
    }
    return ScannerResult { scanner: s, text: "", ok: false };
}

pub struct ScannerResult {
    scanner: Scanner,
    text: String,
    ok: bool,
}
