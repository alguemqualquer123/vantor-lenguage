// Lexicon Standard Library — html.
// Go-parity HTML escaping (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4).
// Import as `import std::html;` (`html::EscapeString`).

/// Escapes `&<>'"` for element and attribute contexts (Go's
/// `html.EscapeString`).
pub fn EscapeString(s: String) -> String {
    let out = s.replace_all("&", "&amp;");
    out = out.replace_all("<", "&lt;");
    out = out.replace_all(">", "&gt;");
    out = out.replace_all("\"", "&quot;");
    out = out.replace_all("'", "&#39;");
    return out;
}

/// Unescapes the entities `EscapeString` produces plus `&apos;`
/// (Go's `html.UnescapeString`, common-entity subset).
pub fn UnescapeString(s: String) -> String {
    let out = s.replace_all("&lt;", "<");
    out = out.replace_all("&gt;", ">");
    out = out.replace_all("&quot;", "\"");
    out = out.replace_all("&#39;", "'");
    out = out.replace_all("&apos;", "'");
    out = out.replace_all("&amp;", "&");
    return out;
}
