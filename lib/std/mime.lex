// Lexicon Standard Library — mime.
// Go-parity media-type helpers (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4).
// Import as `import std::mime;`.

/// Parse result: the media `typ` plus `params` pair list (Go's
/// `mime.ParseMediaType` triple, error-as-`ok`).
pub struct MediaResult {
    typ: String,
    params: Dynamic,
    ok: bool,
}

/// Parses `v` as `type/subtype; k=v; ...` (Go's `mime.ParseMediaType`).
pub fn ParseMediaType(v: String) -> MediaResult {
    let fail = MediaResult { typ: "", params: [], ok: false };
    let semi = Text::index_of(v, ";");
    let head = v;
    let tail = "";
    if semi >= 0 {
        head = Text::slice(v, 0, semi).trim();
        tail = Text::slice(v, semi + 1, v.len());
    } else {
        head = v.trim();
    }
    if Text::index_of(head, "/") < 0 || head == "" {
        return fail;
    }
    let params = [];
    let rest = tail;
    while rest.trim() != "" {
        let ns = Text::index_of(rest, ";");
        let item = rest;
        if ns >= 0 {
            item = Text::slice(rest, 0, ns);
            rest = Text::slice(rest, ns + 1, rest.len());
        } else {
            rest = "";
        }
        let eq = Text::index_of(item, "=");
        if eq >= 0 {
            let k = Text::slice(item, 0, eq).trim().to_lower();
            let val = Text::slice(item, eq + 1, item.len()).trim();
            if val.len() >= 2 && Text::slice(val, 0, 1) == "\"" && Text::slice(val, val.len() - 1, val.len()) == "\"" {
                val = Text::slice(val, 1, val.len() - 1);
            }
            params.push([k, val]);
        }
    }
    return MediaResult { typ: head.to_lower(), params: params, ok: true };
}

/// Formats a type plus params (Go's `mime.FormatMediaType`).
pub fn FormatMediaType(t: String, params: Dynamic) -> String {
    let out = t.to_lower();
    let i = 0;
    while i < params.len() {
        let v = params[i][1].to_string();
        if Text::index_of(v, " ") >= 0 || Text::index_of(v, ";") >= 0 {
            v = "\"" + v + "\"";
        }
        out = out + "; " + params[i][0].to_string() + "=" + v;
        i = i + 1;
    }
    return out;
}

/// Guesses the media type from a file extension (Go's
/// `mime.TypeByExtension`, common-types subset).
pub fn TypeByExtension(ext: String) -> String {
    let e = ext.to_lower();
    if e == ".html" || e == ".htm" {
        return "text/html";
    }
    if e == ".txt" || e == ".md" || e == ".lex" {
        return "text/plain";
    }
    if e == ".css" {
        return "text/css";
    }
    if e == ".js" {
        return "text/javascript";
    }
    if e == ".json" {
        return "application/json";
    }
    if e == ".xml" {
        return "application/xml";
    }
    if e == ".csv" {
        return "text/csv";
    }
    if e == ".png" {
        return "image/png";
    }
    if e == ".jpg" || e == ".jpeg" {
        return "image/jpeg";
    }
    if e == ".gif" {
        return "image/gif";
    }
    if e == ".svg" {
        return "image/svg+xml";
    }
    if e == ".pdf" {
        return "application/pdf";
    }
    if e == ".zip" {
        return "application/zip";
    }
    if e == ".wasm" {
        return "application/wasm";
    }
    return "";
}
