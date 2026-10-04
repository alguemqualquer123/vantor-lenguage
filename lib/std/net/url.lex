// Lexicon Standard Library — net/url.
// Go-parity URL parsing and query codecs
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 5 — entregue mais cedo por ser
// Lex puro). Import as `import std::net::url;`.

/// A parsed URL (Go's `url.URL`, stringly-typed subset).
pub struct URL {
    scheme: String,
    host: String,
    port: String,
    path: String,
    query: String,
    fragment: String,
}

/// Parse result: the `url` plus `ok` (Go's `(u, err)` pair).
pub struct ParseResult {
    url: URL,
    ok: bool,
}

/// Query pair list: `[[key, value], ...]` (Go's `url.Values`).
pub fn ParseQuery(q: String) -> Dynamic {
    let out = [];
    if q == "" {
        return out;
    }
    let parts = q.split("&");
    let i = 0;
    while i < parts.len() {
        let kv = parts[i];
        let eq = Text::index_of(kv, "=");
        if eq < 0 {
            out.push([QueryUnescape(kv), ""]);
        } else {
            out.push([QueryUnescape(Text::slice(kv, 0, eq)), QueryUnescape(Text::slice(kv, eq + 1, kv.len()))]);
        }
        i = i + 1;
    }
    return out;
}

/// Encodes pairs as `k=v&k=v` (Go's `Values.Encode`, insertion order).
pub fn EncodeQuery(pairs: Dynamic) -> String {
    let out = "";
    let i = 0;
    while i < pairs.len() {
        if i > 0 {
            out = out + "&";
        }
        out = out + QueryEscape(pairs[i][0].to_string()) + "=" + QueryEscape(pairs[i][1].to_string());
        i = i + 1;
    }
    return out;
}

/// Parses `raw` (Go's `url.Parse`, hierarchical subset — no userinfo).
pub fn Parse(raw: String) -> ParseResult {
    let fail = ParseResult { url: URL { scheme: "", host: "", port: "", path: "", query: "", fragment: "" }, ok: false };
    if raw == "" {
        return fail;
    }
    let rest = raw;
    let frag = "";
    let fi = Text::index_of(rest, "#");
    if fi >= 0 {
        frag = Text::slice(rest, fi + 1, rest.len());
        rest = Text::slice(rest, 0, fi);
    }
    let query = "";
    let qi = Text::index_of(rest, "?");
    if qi >= 0 {
        query = Text::slice(rest, qi + 1, rest.len());
        rest = Text::slice(rest, 0, qi);
    }
    let scheme = "";
    let sci = Text::index_of(rest, "://");
    if sci >= 0 {
        scheme = Text::slice(rest, 0, sci);
        rest = Text::slice(rest, sci + 3, rest.len());
    }
    let host = rest;
    let path = "";
    let pi = Text::index_of(rest, "/");
    if pi >= 0 {
        host = Text::slice(rest, 0, pi);
        path = Text::slice(rest, pi, rest.len());
    }
    let port = "";
    let ci = Text::last_index_of(host, ":");
    if ci >= 0 {
        port = Text::slice(host, ci + 1, host.len());
        host = Text::slice(host, 0, ci);
    }
    if scheme == "" && host == "" && path == "" {
        return fail;
    }
    return ParseResult { url: URL { scheme: scheme, host: host, port: port, path: path, query: query, fragment: frag }, ok: true };
}

/// Renders the URL back (Go's `u.String()`, canonical subset).
pub fn StringOf(u: URL) -> String {
    let out = "";
    if u.scheme != "" {
        out = out + u.scheme + "://";
    }
    out = out + u.host;
    if u.port != "" {
        out = out + ":" + u.port;
    }
    out = out + u.path;
    if u.query != "" {
        out = out + "?" + u.query;
    }
    if u.fragment != "" {
        out = out + "#" + u.fragment;
    }
    return out;
}

/// Percent-encodes for query components (Go's `url.QueryEscape`):
/// unreserved + a few marks pass through, space becomes `+`.
pub fn QueryEscape(s: String) -> String {
    let out = "";
    let i = 0;
    while i < s.len() {
        let cp = Text::code_at(s, i);
        let c = Text::slice(s, i, i + 1);
        if (c >= "a" && c <= "z") || (c >= "A" && c <= "Z") || (c >= "0" && c <= "9") || c == "-" || c == "_" || c == "." || c == "~" {
            out = out + c;
        } else {
            if c == " " {
                out = out + "+";
            } else {
                out = out + pct_encode(cp);
            }
        }
        i = i + 1;
    }
    return out;
}

/// Decodes `+`/`%XX` (Go's `url.QueryUnescape`; malformed `%` passes
/// through literally, documented).
pub fn QueryUnescape(s: String) -> String {
    let out = "";
    let i = 0;
    while i < s.len() {
        let c = Text::slice(s, i, i + 1);
        if c == "+" {
            out = out + " ";
            i = i + 1;
        } else {
            if c == "%" && i + 2 < s.len() {
                let r = pct_decode(s, i);
                out = out + r[0].to_string();
                i = i + r[1];
            } else {
                out = out + c;
                i = i + 1;
            }
        }
    }
    return out;
}

fn hexval(c: String) -> i64 {
    return Text::index_of("0123456789abcdef", c.to_lower());
}

// Decodes one `%XX` run at `i` (with UTF-8 multi-byte assembly):
// returns [text, chars_consumed].
fn pct_decode(s: String, i: i64) -> Dynamic {
    let b0 = pct_byte(s, i + 1);
    if b0 < 0 {
        return ["%", 1];
    }
    if b0 < 128 {
        return [Text::from_code(b0), 3];
    }
    let need = 1;
    let cp = b0;
    if b0 >= 240 {
        need = 4;
        cp = b0 - 240;
    } else {
        if b0 >= 224 {
            need = 3;
            cp = b0 - 224;
        } else {
            if b0 >= 192 {
                need = 2;
                cp = b0 - 192;
            } else {
                return [Text::from_code(b0), 3];
            }
        }
    }
    let pos = i + 3;
    let k = 1;
    while k < need {
        if pos + 2 >= s.len() || Text::slice(s, pos, pos + 1) != "%" {
            return [Text::from_code(b0), 3];
        }
        let bn = pct_byte(s, pos + 1);
        if bn < 128 || bn >= 192 {
            return [Text::from_code(b0), 3];
        }
        cp = cp * 64 + (bn - 128);
        pos = pos + 3;
        k = k + 1;
    }
    return [Text::from_code(cp), pos - i];
}

fn pct_byte(s: String, j: i64) -> i64 {
    if j + 1 >= s.len() {
        return -1;
    }
    let hi = hexval(Text::slice(s, j, j + 1));
    let lo = hexval(Text::slice(s, j + 1, j + 2));
    if hi < 0 || lo < 0 {
        return -1;
    }
    return hi * 16 + lo;
}

fn hexbyte(b: i64) -> String {
    let digits = "0123456789ABCDEF";
    return "%" + Text::slice(digits, b / 16, b / 16 + 1) + Text::slice(digits, b % 16, b % 16 + 1);
}

// Percent-encodes one code point as its UTF-8 bytes (Go parity for
// non-ASCII input, not just the ASCII fast path).
fn pct_encode(cp: i64) -> String {
    if cp < 128 {
        return hexbyte(cp);
    }
    if cp < 2048 {
        return hexbyte(192 + cp / 64) + hexbyte(128 + cp % 64);
    }
    if cp < 65536 {
        return hexbyte(224 + cp / 4096) + hexbyte(128 + (cp / 64) % 64) + hexbyte(128 + cp % 64);
    }
    return hexbyte(240 + cp / 262144) + hexbyte(128 + (cp / 4096) % 64) + hexbyte(128 + (cp / 64) % 64) + hexbyte(128 + cp % 64);
}
