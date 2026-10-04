// Lexicon Standard Library — net/mail.
// Go-parity mail address parsing (plan: Docs/PLANO_GO_FULL_PARITY.md,
// Fase 5). Display-name + angle-addr grammar (RFC 5322 subset, no groups,
// no encoded words — documented). Import as `import std::net::mail;`.

/// A parsed address (Go's `mail.Address`).
pub struct Address {
    name: String,
    address: String,
}

/// Parse result plus `ok` (Go's `(addr, err)` pair).
pub struct ParseResult {
    addr: Address,
    ok: bool,
}

/// Parses `s` as `Name <addr>` or bare `addr` (Go's `mail.ParseAddress`).
pub fn ParseAddress(s: String) -> ParseResult {
    let fail = ParseResult { addr: Address { name: "", address: "" }, ok: false };
    let t = s.trim();
    if t == "" {
        return fail;
    }
    let lt = Text::index_of(t, "<");
    if lt < 0 {
        if valid_addr(t) {
            return ParseResult { addr: Address { name: "", address: t }, ok: true };
        }
        return fail;
    }
    let gt = Text::index_of(t, ">");
    if gt < 0 || gt < lt {
        return fail;
    }
    let name = Text::slice(t, 0, lt).trim();
    if name.len() >= 2 && Text::slice(name, 0, 1) == "\"" && Text::slice(name, name.len() - 1, name.len()) == "\"" {
        name = Text::slice(name, 1, name.len() - 1);
    }
    let addr = Text::slice(t, lt + 1, gt).trim();
    if !valid_addr(addr) {
        return fail;
    }
    if Text::slice(t, gt + 1, t.len()).trim() != "" {
        return fail;
    }
    return ParseResult { addr: Address { name: name, address: addr }, ok: true };
}

/// Parses a comma-separated list (Go's `mail.ParseAddressList`).
pub fn ParseAddressList(s: String) -> Dynamic {
    let out = [];
    let cur = "";
    let depth = 0;
    let i = 0;
    while i < s.len() {
        let c = Text::slice(s, i, i + 1);
        if c == "\"" {
            cur = cur + c;
            i = i + 1;
            while i < s.len() && Text::slice(s, i, i + 1) != "\"" {
                cur = cur + Text::slice(s, i, i + 1);
                i = i + 1;
            }
            if i < s.len() {
                cur = cur + "\"";
                i = i + 1;
            }
            continue;
        }
        if c == "<" {
            depth = depth + 1;
        }
        if c == ">" && depth > 0 {
            depth = depth - 1;
        }
        if c == "," && depth == 0 {
            let r = ParseAddress(cur);
            if !r.ok {
                return [];
            }
            out.push(r.addr);
            cur = "";
            i = i + 1;
            continue;
        }
        cur = cur + c;
        i = i + 1;
    }
    if cur.trim() != "" {
        let r = ParseAddress(cur);
        if !r.ok {
            return [];
        }
        out.push(r.addr);
    }
    return out;
}

/// Renders back (Go's `Address.String()`).
pub fn StringOf(a: Address) -> String {
    if a.name == "" {
        return a.address;
    }
    return a.name + " <" + a.address + ">";
}

fn valid_addr(a: String) -> bool {
    let at = Text::index_of(a, "@");
    if at <= 0 || at >= a.len() - 1 {
        return false;
    }
    if Text::index_of(a, " ") >= 0 || Text::index_of(a, "<") >= 0 || Text::index_of(a, ">") >= 0 {
        return false;
    }
    return true;
}
