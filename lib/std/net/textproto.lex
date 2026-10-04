// Lexicon Standard Library — net/textproto.
// Go-parity text-protocol reading (plan: Docs/PLANO_GO_FULL_PARITY.md,
// Fase 5). Headers parse from a string (`MIMEHeader` = pair list with
// canonical keys); dot-stuffing and truncation are caller's job.
// Import as `import std::net::textproto;`.

/// Read result: header pairs plus the remaining `body`.
pub struct HeaderResult {
    header: Dynamic,
    body: String,
}

/// Reads `Key: value` lines until the blank line (Go's
/// `textproto.Reader.ReadMIMEHeader`, string-adapted).
pub fn ReadMIMEHeader(s: String) -> HeaderResult {
    let header = [];
    let lines = s.split("\n");
    let i = 0;
    let cur = "";
    while i < lines.len() {
        let ln = lines[i];
        if ln.len() > 0 && Text::slice(ln, ln.len() - 1, ln.len()) == "\r" {
            ln = Text::slice(ln, 0, ln.len() - 1);
        }
        if ln == "" {
            if cur != "" {
                header = add_field(header, cur);
                cur = "";
            }
            let body = [];
            let j = i + 1;
            while j < lines.len() {
                body.push(lines[j]);
                j = j + 1;
            }
            return HeaderResult { header: header, body: Text::join(body, "\n") };
        }
        if (Text::slice(ln, 0, 1) == " " || Text::slice(ln, 0, 1) == "\t") && cur != "" {
            cur = cur + " " + ln.trim();
        } else {
            if cur != "" {
                header = add_field(header, cur);
            }
            cur = ln;
        }
        i = i + 1;
    }
    if cur != "" {
        header = add_field(header, cur);
    }
    return HeaderResult { header: header, body: "" };
}

fn add_field(header: Dynamic, ln: String) -> Dynamic {
    let ci = Text::index_of(ln, ":");
    if ci < 0 {
        return header;
    }
    let k = CanonicalKey(Text::slice(ln, 0, ci).trim());
    let v = Text::slice(ln, ci + 1, ln.len()).trim();
    header.push([k, v]);
    return header;
}

/// Canonicalizes `MIME-Header-Key` form (Go's `textproto.CanonicalMIMEHeaderKey`).
pub fn CanonicalKey(s: String) -> String {
    let out = "";
    let up = true;
    let i = 0;
    while i < s.len() {
        let c = Text::slice(s, i, i + 1);
        if c == "-" {
            out = out + "-";
            up = true;
        } else {
            if up {
                out = out + c.to_upper();
            } else {
                out = out + c.to_lower();
            }
            up = false;
        }
        i = i + 1;
    }
    return out;
}

/// Trims trailing CRLF (Go's `textproto.TrimString`).
pub fn TrimString(s: String) -> String {
    let e = s.len();
    while e > 0 {
        let c = Text::slice(s, e - 1, e);
        if c == "\r" || c == "\n" || c == " " || c == "\t" {
            e = e - 1;
        } else {
            break;
        }
    }
    return Text::slice(s, 0, e);
}

/// Looks up the first value for `key` (canonicalized).
pub fn Get(header: Dynamic, key: String) -> String {
    let ck = CanonicalKey(key);
    let i = 0;
    while i < header.len() {
        if header[i][0] == ck {
            return header[i][1].to_string();
        }
        i = i + 1;
    }
    return "";
}
