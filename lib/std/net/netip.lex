// Lexicon Standard Library — net/netip.
// Go-parity IP address / prefix / addr:port parsing and formatting
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 5). Addresses are plain Lex
// values — `NetIP { octets, ok }` holds 4 bytes for IPv4 or 16 for IPv6 —
// so nothing here mutates shared state. Three documented deviations from
// `net/netip`: `::ffff:a.b.c.d` is normalised to bare IPv4 on parse (Go's
// `Addr.Unmap`), v6 output always uses RFC 5952 hex groups (a parsed
// `::a.b.c.d` prints as `::102:304`), and interface zones
// (`fe80::1%eth0`) are rejected. Import as `import std::net::netip;`.

/// A parsed IP address: 4 octets (v4) or 16 (v6), `ok` = Go's `err == nil`.
pub struct NetIP {
    octets: Dynamic,
    ok: bool,
}

/// Network prefix (Go's `netip.Prefix`).
pub struct NetPrefix {
    ip: NetIP,
    bits: i64,
    ok: bool,
}

/// Address plus port (Go's `netip.AddrPort`).
pub struct NetAddrPort {
    ip: NetIP,
    port: i64,
    ok: bool,
}

const POW8 = [1, 2, 4, 8, 16, 32, 64, 128, 256];

/// Parses v4 or v6 text (Go's `netip.ParseIP`).
pub fn ParseIP(s: String) -> NetIP {
    if Text::index_of(s, ":") < 0 {
        return parse_v4(s);
    }
    return parse_v6(s);
}

/// True when `s` is a valid address (Go's `netip.ParseIP` + nil check).
pub fn IsIP(s: String) -> bool {
    return ParseIP(s).ok;
}

/// Textual form (Go's `Addr.String`).
pub fn ToString(ip: NetIP) -> String {
    if ip.octets.len() == 4 {
        return v4_text(ip.octets);
    }
    if ip.octets.len() == 16 {
        return v6_text(ip.octets);
    }
    return "";
}

/// Bytes as parsed (Go's `Addr.As4` / `As16`, list-adapted).
pub fn AsBytes(ip: NetIP) -> Dynamic {
    return ip.octets;
}

/// Always the 16-byte form (Go's `Addr.As16`).
pub fn As16(ip: NetIP) -> Dynamic {
    let out = [];
    if ip.octets.len() == 4 {
        let i = 0;
        while i < 10 {
            out.push(0);
            i = i + 1;
        }
        out.push(255);
        out.push(255);
        i = 0;
        while i < 4 {
            out.push(ip.octets[i]);
            i = i + 1;
        }
        return out;
    }
    if ip.octets.len() == 16 {
        let i = 0;
        while i < 16 {
            out.push(ip.octets[i]);
            i = i + 1;
        }
    }
    return out;
}

/// Builds an address from 4 or 16 octets (Go's `AddrFromSlice`).
pub fn FromBytes(b: Dynamic) -> NetIP {
    let bad = NetIP { octets: [], ok: false };
    if b.len() != 4 && b.len() != 16 {
        return bad;
    }
    let out = [];
    let i = 0;
    while i < b.len() {
        let v = b[i];
        if v < 0 || v > 255 {
            return bad;
        }
        out.push(v);
        i = i + 1;
    }
    return NetIP { octets: out, ok: true };
}

pub fn Is4(ip: NetIP) -> bool {
    return ip.ok && ip.octets.len() == 4;
}

pub fn Is6(ip: NetIP) -> bool {
    return ip.ok;
}

/// Address width in bits: 32 or 128 (Go's `Addr.BitLen`).
pub fn Bits(ip: NetIP) -> i64 {
    if ip.octets.len() == 16 {
        return 128;
    }
    if ip.octets.len() == 4 {
        return 32;
    }
    return 0;
}

/// 0.0.0.0 or :: (Go's `Addr.IsUnspecified`).
pub fn IsUnspecified(ip: NetIP) -> bool {
    if ip.ok == false {
        return false;
    }
    let i = 0;
    while i < ip.octets.len() {
        if ip.octets[i] != 0 {
            return false;
        }
        i = i + 1;
    }
    return true;
}

/// 127.0.0.0/8 or ::1 (Go's `Addr.IsLoopback`).
pub fn IsLoopback(ip: NetIP) -> bool {
    if ip.octets.len() == 4 {
        return ip.octets[0] == 127;
    }
    if ip.octets.len() == 16 {
        let i = 0;
        while i < 15 {
            if ip.octets[i] != 0 {
                return false;
            }
            i = i + 1;
        }
        return ip.octets[15] == 1;
    }
    return false;
}

/// 224.0.0.0/4 or ff00::/8 (Go's `Addr.IsMulticast`).
pub fn IsMulticast(ip: NetIP) -> bool {
    if ip.octets.len() == 4 {
        return ip.octets[0] >= 224 && ip.octets[0] <= 239;
    }
    if ip.octets.len() == 16 {
        return ip.octets[0] == 255;
    }
    return false;
}

/// fe80::/10 (Go's `Addr.IsLinkLocalUnicast`).
pub fn IsLinkLocalUnicast(ip: NetIP) -> bool {
    if ip.octets.len() == 16 {
        return ip.octets[0] == 254 && (ip.octets[1] & 192) == 128;
    }
    if ip.octets.len() == 4 {
        return ip.octets[0] == 169 && ip.octets[1] == 254;
    }
    return false;
}

/// Parses `addr/bits` (Go's `netip.ParsePrefix`).
pub fn ParsePrefix(s: String) -> NetPrefix {
    let bad = NetPrefix { ip: NetIP { octets: [], ok: false }, bits: 0, ok: false };
    let slash = Text::last_index_of(s, "/");
    if slash < 0 {
        return bad;
    }
    let ip = ParseIP(Text::slice(s, 0, slash));
    if ip.ok == false {
        return bad;
    }
    let bits = num_val(Text::slice(s, slash + 1, s.len()), 3, 128);
    if bits < 0 {
        return bad;
    }
    if bits > Bits(ip) {
        return bad;
    }
    return NetPrefix { ip: ip, bits: bits, ok: true };
}

/// `addr/bits` (Go's `Prefix.String`).
pub fn PrefixString(p: NetPrefix) -> String {
    if p.ok == false {
        return "";
    }
    return ToString(p.ip) + "/" + dec(p.bits);
}

/// Network half of the prefix, host bits cleared (Go's `Prefix.Masked`).
pub fn Masked(p: NetPrefix) -> NetPrefix {
    if p.ok == false {
        return p;
    }
    let a = p.ip.octets;
    let full = p.bits / 8;
    let rem = p.bits % 8;
    let out = [];
    let i = 0;
    while i < full {
        out.push(a[i]);
        i = i + 1;
    }
    if rem > 0 {
        let d = 256 / POW8[rem];
        out.push(a[full] / d * d);
    }
    while out.len() < a.len() {
        out.push(0);
    }
    return NetPrefix { ip: NetIP { octets: out, ok: true }, bits: p.bits, ok: true };
}

/// True when `ip` is inside `p` (Go's `Prefix.Contains`).
pub fn Contains(p: NetPrefix, ip: NetIP) -> bool {
    if p.ok == false || ip.ok == false {
        return false;
    }
    let a = p.ip.octets;
    let b = ip.octets;
    if a.len() != b.len() {
        return false;
    }
    let full = p.bits / 8;
    let i = 0;
    while i < full {
        if a[i] != b[i] {
            return false;
        }
        i = i + 1;
    }
    let rem = p.bits % 8;
    if rem > 0 {
        let d = 256 / POW8[rem];
        if a[i] / d != b[i] / d {
            return false;
        }
    }
    return true;
}

/// Parses `1.2.3.4:80` or `[::1]:80` (Go's `netip.ParseAddrPort`).
pub fn ParseAddrPort(s: String) -> NetAddrPort {
    let bad = NetAddrPort { ip: NetIP { octets: [], ok: false }, port: 0, ok: false };
    let host = s;
    if Text::starts_with(s, "[") {
        let close = Text::index_of(s, "]");
        if close < 0 {
            return bad;
        }
        host = Text::slice(s, 1, close);
        let rest = Text::slice(s, close + 1, s.len());
        if Text::starts_with(rest, ":") == false {
            return bad;
        }
        let port = num_val(Text::slice(rest, 1, rest.len()), 5, 65535);
        let ip = ParseIP(host);
        if ip.ok == false || port < 0 {
            return bad;
        }
        return NetAddrPort { ip: ip, port: port, ok: true };
    }
    let colon = Text::last_index_of(s, ":");
    if colon < 0 {
        return bad;
    }
    let port = num_val(Text::slice(s, colon + 1, s.len()), 5, 65535);
    let ip = ParseIP(Text::slice(s, 0, colon));
    if ip.ok == false || port < 0 {
        return bad;
    }
    return NetAddrPort { ip: ip, port: port, ok: true };
}

/// `addr:port`, bracketed for v6 (Go's `AddrPort.String`).
pub fn AddrPortString(ap: NetAddrPort) -> String {
    if ap.ok == false {
        return "";
    }
    if ap.ip.octets.len() == 16 {
        return "[" + ToString(ap.ip) + "]:" + dec(ap.port);
    }
    return ToString(ap.ip) + ":" + dec(ap.port);
}

// ---- internals -----------------------------------------------------------

// Dotted quad; leading zeros are rejected like Go's `ParseIPv4`.
fn parse_v4(s: String) -> NetIP {
    let bad = NetIP { octets: [], ok: false };
    let parts = s.split(".");
    if parts.len() != 4 {
        return bad;
    }
    let out = [];
    let i = 0;
    while i < 4 {
        let v = octet(parts[i]);
        if v < 0 {
            return bad;
        }
        out.push(v);
        i = i + 1;
    }
    return NetIP { octets: out, ok: true };
}

fn octet(p: String) -> i64 {
    let bad = -1;
    if p.len() < 1 || p.len() > 3 {
        return bad;
    }
    if p.len() > 1 && Text::starts_with(p, "0") {
        return bad;
    }
    let v = 0;
    let i = 0;
    while i < p.len() {
        let c = Text::code_at(p, i);
        if c < 48 || c > 57 {
            return bad;
        }
        v = v * 10 + (c - 48);
        i = i + 1;
    }
    if v > 255 {
        return bad;
    }
    return v;
}

// v6: `::` compression plus an optional trailing dotted quad.
fn parse_v6(s: String) -> NetIP {
    let bad = NetIP { octets: [], ok: false };
    let dbl = Text::index_of(s, "::");
    if dbl >= 0 {
        if Text::index_of(Text::slice(s, dbl + 1, s.len()), "::") >= 0 {
            return bad;
        }
        if Text::starts_with(s, ":") && Text::code_at(s, 1) != 58 {
            return bad;
        }
    }
    let compressed = dbl >= 0;
    let head = s;
    let tail = "";
    if compressed {
        head = Text::slice(s, 0, dbl);
        tail = Text::slice(s, dbl + 2, s.len());
    }
    let hp = [];
    if head.len() > 0 {
        hp = head.split(":");
    }
    let tp = [];
    if tail.len() > 0 {
        tp = tail.split(":");
    }
    let head_v4_at = hp.len() - 1;
    if compressed {
        head_v4_at = -1;
    }
    let g1 = side_groups(hp, head_v4_at);
    if g1.len() > 0 && g1[0] < 0 {
        return bad;
    }
    let g2 = side_groups(tp, tp.len() - 1);
    if g2.len() > 0 && g2[0] < 0 {
        return bad;
    }
    let total = g1.len() + g2.len();
    if compressed {
        if total > 7 {
            return bad;
        }
    } else if total != 8 {
        return bad;
    }
    let bytes = [];
    let i = 0;
    while i < g1.len() {
        bytes.push(g1[i] / 256);
        bytes.push(g1[i] % 256);
        i = i + 1;
    }
    let z = total;
    while z < 8 {
        bytes.push(0);
        bytes.push(0);
        z = z + 1;
    }
    i = 0;
    while i < g2.len() {
        bytes.push(g2[i] / 256);
        bytes.push(g2[i] % 256);
        i = i + 1;
    }
    if bytes.len() != 16 {
        return bad;
    }
    // ::ffff:a.b.c.d is the same host as a.b.c.d (Go's `Unmap`).
    if head_only_zero(bytes, 10) && bytes[10] == 255 && bytes[11] == 255 {
        return NetIP { octets: [bytes[12], bytes[13], bytes[14], bytes[15]], ok: true };
    }
    return NetIP { octets: bytes, ok: true };
}

fn head_only_zero(b: Dynamic, n: i64) -> bool {
    let i = 0;
    while i < n {
        if b[i] != 0 {
            return false;
        }
        i = i + 1;
    }
    return true;
}

// Parses one side of the `::`; a dotted quad is only allowed at
// `v4_at`. Failures come back as `[-1]`.
fn side_groups(pieces: Dynamic, v4_at: i64) -> Dynamic {
    let out = [];
    let i = 0;
    while i < pieces.len() {
        let p = pieces[i];
        if Text::index_of(p, ".") >= 0 {
            if i != v4_at {
                return [-1];
            }
            let q = v4_octets(p);
            if q.len() != 4 {
                return [-1];
            }
            out.push(q[0] * 256 + q[1]);
            out.push(q[2] * 256 + q[3]);
        } else {
            let v = hex_group(p);
            if v < 0 {
                return [-1];
            }
            out.push(v);
        }
        i = i + 1;
    }
    return out;
}

fn v4_octets(s: String) -> Dynamic {
    let parts = s.split(".");
    if parts.len() != 4 {
        return [];
    }
    let out = [];
    let i = 0;
    while i < 4 {
        let v = octet(parts[i]);
        if v < 0 {
            return [];
        }
        out.push(v);
        i = i + 1;
    }
    return out;
}

fn hex_group(p: String) -> i64 {
    if p.len() < 1 || p.len() > 4 {
        return -1;
    }
    let v = 0;
    let i = 0;
    while i < p.len() {
        let d = hex_val(Text::slice(p, i, i + 1));
        if d < 0 {
            return -1;
        }
        v = v * 16 + d;
        i = i + 1;
    }
    return v;
}

fn hex_val(c: String) -> i64 {
    return Text::index_of("0123456789abcdef", c.to_lower());
}

fn v4_text(b: Dynamic) -> String {
    return dec(b[0]) + "." + dec(b[1]) + "." + dec(b[2]) + "." + dec(b[3]);
}

// RFC 5952 shape: lowercase, no leading zeros, longest zero run as `::`.
fn v6_text(b: Dynamic) -> String {
    if head_only_zero(b, 16) {
        return "::";
    }
    let g = [];
    let i = 0;
    while i < 16 {
        g.push(b[i] * 256 + b[i + 1]);
        i = i + 2;
    }
    let best_start = -1;
    let best_len = 0;
    i = 0;
    while i < 8 {
        if g[i] == 0 {
            let j = i;
            while j < 8 && g[j] == 0 {
                j = j + 1;
            }
            if j - i > best_len {
                best_start = i;
                best_len = j - i;
            }
            i = j;
        } else {
            i = i + 1;
        }
    }
    if best_len < 2 {
        best_start = -1;
    }
    let out = "";
    i = 0;
    while i < 8 {
        if i == best_start {
            out = out + "::";
            i = i + best_len;
            while i < 8 {
                out = out + group_hex(g[i]);
                i = i + 1;
                if i < 8 {
                    out = out + ":";
                }
            }
            return out;
        }
        out = out + group_hex(g[i]);
        i = i + 1;
        if i < 8 && i != best_start {
            out = out + ":";
        }
    }
    return out;
}

fn group_hex(v: i64) -> String {
    if v == 0 {
        return "0";
    }
    let digits = "0123456789abcdef";
    let out = "";
    let n = v;
    while n > 0 {
        out = Text::slice(digits, n % 16, n % 16 + 1) + out;
        n = n / 16;
    }
    return out;
}

fn dec(v: i64) -> String {
    if v == 0 {
        return "0";
    }
    let digits = "0123456789";
    let out = "";
    let n = v;
    while n > 0 {
        out = Text::slice(digits, n % 10, n % 10 + 1) + out;
        n = n / 10;
    }
    return out;
}

// Unsigned decimal with Go's limits: no leading zeros, at most `max_len`
// digits, value at most `max`; -1 on failure.
fn num_val(s: String, max_len: i64, max: i64) -> i64 {
    if s.len() < 1 || s.len() > max_len {
        return -1;
    }
    if s.len() > 1 && Text::starts_with(s, "0") {
        return -1;
    }
    let v = 0;
    let i = 0;
    while i < s.len() {
        let c = Text::code_at(s, i);
        if c < 48 || c > 57 {
            return -1;
        }
        v = v * 10 + (c - 48);
        i = i + 1;
    }
    if v > max {
        return -1;
    }
    return v;
}
