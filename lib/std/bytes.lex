// Lexicon Standard Library — bytes.
// Go-parity byte-slice helpers (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
// Bytes travel as `Dynamic` lists of i64 in [0, 255] (there is no addressable
// `u8` array in the interpreter yet); ASCII string bridges use the native
// `Text::*` builtins so conversions stay O(n).

/// Builds a byte list from the ASCII bytes of `s`.
pub fn FromString(s: String) -> Dynamic {
    let out = [];
    let i = 0;
    let n = s.len();
    while i < n {
        out.push(Text::code_at(s, i));
        i = i + 1;
    }
    return out;
}

/// Renders a byte list as a string (one code point per byte; exact for
/// ASCII, which is the documented fast path).
pub fn ToString(b: Dynamic) -> String {
    let out = "";
    let i = 0;
    let n = b.len();
    while i < n {
        out = out + Text::from_code(b[i]);
        i = i + 1;
    }
    return out;
}

/// Reports whether `a` and `b` hold the same bytes (Go's `bytes.Equal`).
pub fn Equal(a: Dynamic, b: Dynamic) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let i = 0;
    let n = a.len();
    while i < n {
        if a[i] != b[i] {
            return false;
        }
        i = i + 1;
    }
    return true;
}

/// Three-way comparison of two byte lists (Go's `bytes.Compare`).
pub fn Compare(a: Dynamic, b: Dynamic) -> i64 {
    let n = a.len();
    let m = b.len();
    let k = m;
    if n < m {
        k = n;
    }
    let i = 0;
    while i < k {
        if a[i] < b[i] {
            return -1;
        }
        if a[i] > b[i] {
            return 1;
        }
        i = i + 1;
    }
    if n < m {
        return -1;
    }
    if n > m {
        return 1;
    }
    return 0;
}

/// Reports whether `sub` occurs in `b` (Go's `bytes.Contains`).
pub fn Contains(b: Dynamic, sub: Dynamic) -> bool {
    return Index(b, sub) >= 0;
}

/// Returns the first index of `sub` in `b`, or -1 (Go's `bytes.Index`).
pub fn Index(b: Dynamic, sub: Dynamic) -> i64 {
    let n = b.len();
    let m = sub.len();
    if m == 0 {
        return 0;
    }
    if m > n {
        return -1;
    }
    let i = 0;
    while i <= n - m {
        let hit = true;
        let j = 0;
        while j < m {
            if b[i + j] != sub[j] {
                hit = false;
                break;
            }
            j = j + 1;
        }
        if hit {
            return i;
        }
        i = i + 1;
    }
    return -1;
}

/// Reports whether `b` starts with `prefix` (Go's `bytes.HasPrefix`).
pub fn HasPrefix(b: Dynamic, prefix: Dynamic) -> bool {
    if prefix.len() > b.len() {
        return false;
    }
    let i = 0;
    while i < prefix.len() {
        if b[i] != prefix[i] {
            return false;
        }
        i = i + 1;
    }
    return true;
}

/// Reports whether `b` ends with `suffix` (Go's `bytes.HasSuffix`).
pub fn HasSuffix(b: Dynamic, suffix: Dynamic) -> bool {
    let n = b.len();
    let m = suffix.len();
    if m > n {
        return false;
    }
    let i = 0;
    while i < m {
        if b[n - m + i] != suffix[i] {
            return false;
        }
        i = i + 1;
    }
    return true;
}

/// Concatenates `elems` with `sep` between them (Go's `bytes.Join`).
/// Elements are byte lists; the result is a byte list.
pub fn Join(elems: Dynamic, sep: Dynamic) -> Dynamic {
    let out = [];
    let i = 0;
    let n = elems.len();
    while i < n {
        if i > 0 {
            let k = 0;
            while k < sep.len() {
                out.push(sep[k]);
                k = k + 1;
            }
        }
        let part = elems[i];
        let j = 0;
        while j < part.len() {
            out.push(part[j]);
            j = j + 1;
        }
        i = i + 1;
    }
    return out;
}

/// Returns `count` copies of `b` concatenated (Go's `bytes.Repeat`).
pub fn Repeat(b: Dynamic, count: i64) -> Dynamic {
    let out = [];
    let i = 0;
    while i < count {
        let j = 0;
        while j < b.len() {
            out.push(b[j]);
            j = j + 1;
        }
        i = i + 1;
    }
    return out;
}

/// Splits `b` around each instance of `sep` (Go's `bytes.Split`).
pub fn Split(b: Dynamic, sep: Dynamic) -> Dynamic {
    let out = [];
    let cur = [];
    let i = 0;
    let n = b.len();
    let m = sep.len();
    while i < n {
        if m > 0 && i + m <= n && SliceEq(b, i, sep) {
            out.push(cur);
            cur = [];
            i = i + m;
        } else {
            cur.push(b[i]);
            i = i + 1;
        }
    }
    out.push(cur);
    return out;
}

fn SliceEq(b: Dynamic, at: i64, sep: Dynamic) -> bool {
    let j = 0;
    while j < sep.len() {
        if b[at + j] != sep[j] {
            return false;
        }
        j = j + 1;
    }
    return true;
}

/// Returns `b` upper-cased ASCII (Go's `bytes.ToUpper`).
pub fn ToUpper(b: Dynamic) -> Dynamic {
    return FromString(ToString(b).to_upper());
}

/// Returns `b` lower-cased ASCII (Go's `bytes.ToLower`).
pub fn ToLower(b: Dynamic) -> Dynamic {
    return FromString(ToString(b).to_lower());
}

/// Returns `b` without leading/trailing ASCII white space
/// (Go's `bytes.TrimSpace`).
pub fn TrimSpace(b: Dynamic) -> Dynamic {
    return FromString(ToString(b).trim());
}
