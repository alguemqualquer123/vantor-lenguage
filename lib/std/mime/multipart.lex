// Lexicon Standard Library — mime/multipart.
// Go-parity multipart parsing, string-adapted
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4). Part headers reuse the
// `net/textproto` shape (`[[Key, value], ...]` with canonical keys).
// Import as `import std::mime::multipart;`.

import std::net::textproto;

/// One part: headers plus raw `body` (Go's `multipart.Part`, buffered).
pub struct Part {
    header: Dynamic,
    body: String,
}

/// Parse result plus `ok` (Go's `(parts, err)` pair).
pub struct ParseResult {
    parts: Dynamic,
    ok: bool,
}

/// Splits `s` on `--boundary` delimiters (Go's `multipart.Reader`,
/// document-adapted: preamble ignored, epilogue dropped).
pub fn Parse(s: String, boundary: String) -> ParseResult {
    let fail = ParseResult { parts: [], ok: false };
    if boundary == "" {
        return fail;
    }
    let delim = "--" + boundary;
    let chunks = s.split(delim);
    if chunks.len() < 2 {
        return fail;
    }
    let parts = [];
    let i = 1;
    while i < chunks.len() {
        let c = chunks[i];
        if c.len() >= 2 && Text::slice(c, 0, 2) == "--" {
            break;
        }
        let body = c;
        if body.len() > 0 && Text::slice(body, 0, 1) == "\r" {
            body = Text::slice(body, 1, body.len());
        }
        if body.len() > 0 && Text::slice(body, 0, 1) == "\n" {
            body = Text::slice(body, 1, body.len());
        }
        if body.len() >= 2 && Text::slice(body, body.len() - 2, body.len()) == "\r\n" {
            body = Text::slice(body, 0, body.len() - 2);
        } else {
            if body.len() >= 1 && Text::slice(body, body.len() - 1, body.len()) == "\n" {
                body = Text::slice(body, 0, body.len() - 1);
            }
        }
        let hr = textproto::ReadMIMEHeader(body);
        parts.push(Part { header: hr.header, body: hr.body });
        i = i + 1;
    }
    return ParseResult { parts: parts, ok: true };
}

/// Returns the first header value for `key` on a part (canonicalized).
pub fn HeaderGet(p: Part, key: String) -> String {
    return textproto::Get(p.header, key);
}

/// Returns the filename parameter of Content-Disposition, or "".
pub fn FileName(p: Part) -> String {
    let cd = textproto::Get(p.header, "Content-Disposition");
    let fi = Text::index_of(cd, "filename=");
    if fi < 0 {
        return "";
    }
    let v = Text::slice(cd, fi + 9, cd.len()).trim();
    if v.len() >= 2 && Text::slice(v, 0, 1) == "\"" {
        let end = Text::index_of(Text::slice(v, 1, v.len()), "\"");
        if end >= 0 {
            return Text::slice(v, 1, end + 1);
        }
    }
    let sp = Text::index_of(v, ";");
    if sp >= 0 {
        return Text::slice(v, 0, sp).trim();
    }
    return v;
}
