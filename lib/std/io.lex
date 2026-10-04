// Lexicon Standard Library — io.
// Go-parity streaming primitives (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
//
// Value-semantics note (same as `sort`): structs and lists pass by copy
// across calls, so readers/writers are *functional* — every advancing
// operation returns the updated value alongside its result:
// `let rr = io::Read(r, 5); r = rr.reader;`.

/// Sentinel reported when no bytes remain (Go's `io.EOF`).
pub const EOF = "EOF";

/// An in-memory byte stream (Go's `io.Reader` over a string).
pub struct Reader {
    data: String,
    pos: i64,
}

/// A growing sink (Go's `io.Writer` over a string buffer).
pub struct Writer {
    buf: String,
}

/// Result of a read: the advanced `reader`, the `chunk` and the `eof`
/// flag (Go's `(n, err)` pair, struct-adapted).
pub struct ReadResult {
    reader: Reader,
    chunk: String,
    eof: bool,
}

/// Result of a write: the advanced `writer` and the byte count.
pub struct WriteResult {
    writer: Writer,
    n: i64,
}

/// Result of a copy: both advanced values and the character count.
pub struct CopyResult {
    writer: Writer,
    reader: Reader,
    n: i64,
}

/// Builds a reader over `data` (Go's `strings.NewReader`).
pub fn NewReader(data: String) -> Reader {
    return Reader { data: data, pos: 0 };
}

/// Builds an empty sink.
pub fn NewWriter() -> Writer {
    return Writer { buf: "" };
}

/// Reads up to `n` characters, returning the advanced reader. The final
/// piece arrives with `eof == true` (Go's `Read` with `io.EOF`).
pub fn Read(r: Reader, n: i64) -> ReadResult {
    let rest = r.data.len() - r.pos;
    if rest <= 0 {
        return ReadResult { reader: r, chunk: "", eof: true };
    }
    let take = n;
    if take > rest {
        take = rest;
    }
    let chunk = Text::slice(r.data, r.pos, r.pos + take);
    let next = Reader { data: r.data, pos: r.pos + take };
    return ReadResult { reader: next, chunk: chunk, eof: next.pos >= next.data.len() };
}

/// Reads everything left in `r` (Go's `io.ReadAll`).
pub fn ReadAll(r: Reader) -> String {
    let pos = r.pos;
    return Text::slice(r.data, pos, r.data.len());
}

/// Appends `s` to the sink, returning the advanced writer plus the byte
/// count (Go's `Write`).
pub fn Write(w: Writer, s: String) -> WriteResult {
    let next = Writer { buf: w.buf + s };
    return WriteResult { writer: next, n: s.len() };
}

/// Copies all of `r` into `w` (Go's `io.Copy`).
pub fn Copy(w: Writer, r: Reader) -> CopyResult {
    let tail = Text::slice(r.data, r.pos, r.data.len());
    let next = Writer { buf: w.buf + tail };
    let done = Reader { data: r.data, pos: r.data.len() };
    return CopyResult { writer: next, reader: done, n: tail.len() };
}

/// Discards at most `n` characters, returning the advanced reader plus
/// the skipped count (Go's `io.CopyN` into `io.Discard`).
pub fn Discard(r: Reader, n: i64) -> ReadResult {
    let left = r.data.len() - r.pos;
    if left <= 0 {
        return ReadResult { reader: r, chunk: "", eof: true };
    }
    let take = n;
    if take > left {
        take = left;
    }
    let next = Reader { data: r.data, pos: r.pos + take };
    return ReadResult { reader: next, chunk: "", eof: next.pos >= next.data.len() };
}
