// Lexicon Standard Library — strings.
// Go-parity string helpers (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1).
// All functions are pure; arguments follow Go's package-function style.

/// Returns the number of characters (runes) in `s`.
pub fn RuneCount(s: String) -> i64 {
    return s.len();
}

/// Reports whether `substr` is contained within `s`.
pub fn Contains(s: String, substr: String) -> bool {
    return s.contains(substr);
}

/// Reports whether `s` begins with `prefix` (native, O(n)).
pub fn HasPrefix(s: String, prefix: String) -> bool {
    return Text::starts_with(s, prefix);
}

/// Reports whether `s` ends with `suffix` (native, O(n)).
pub fn HasSuffix(s: String, suffix: String) -> bool {
    return Text::ends_with(s, suffix);
}

/// Returns the substring `s[begin:end]` (character offsets, end exclusive;
/// native, O(n) — clamps out-of-range bounds like `Text::slice`).
pub fn Slice(s: String, begin: i64, end: i64) -> String {
    return Text::slice(s, begin, end);
}

/// Returns `s` with all characters upper-cased.
pub fn ToUpper(s: String) -> String {
    return s.to_upper();
}

/// Returns `s` with all characters lower-cased.
pub fn ToLower(s: String) -> String {
    return s.to_lower();
}

/// Returns a new string consisting of `count` copies of `s` (native).
pub fn Repeat(s: String, count: i64) -> String {
    return Text::repeat(s, count);
}

/// Concatenates the elements of `elems` with `sep` between them (native).
pub fn Join(elems: Dynamic, sep: String) -> String {
    return Text::join(elems, sep);
}

/// Splits `s` around each instance of `sep`, returning the list of parts.
pub fn Split(s: String, sep: String) -> Dynamic {
    return s.split(sep);
}

/// Returns a copy of `s` with every instance of `from` replaced by `to`.
pub fn ReplaceAll(s: String, from: String, to: String) -> String {
    return s.replace_all(from, to);
}

/// Returns `s` with all leading and trailing white space removed.
pub fn TrimSpace(s: String) -> String {
    return s.trim();
}

/// Returns the index of the first instance of `substr` in `s`, or -1
/// (native, byte-search with char-offset result — Go's `strings.Index`).
pub fn Index(s: String, substr: String) -> i64 {
    return Text::index_of(s, substr);
}

/// Returns the index of the last instance of `substr` in `s`, or -1
/// (Go's `strings.LastIndex`).
pub fn LastIndex(s: String, substr: String) -> i64 {
    return Text::last_index_of(s, substr);
}

/// Returns the number of (possibly overlapping) instances of `substr` in
/// `s` (Go's `strings.Count`; empty `substr` counts 1 + rune count).
pub fn Count(s: String, substr: String) -> i64 {
    if substr == "" {
        return s.len() + 1;
    }
    let n = 0;
    let rest = s;
    while true {
        let i = Text::index_of(rest, substr);
        if i < 0 {
            return n;
        }
        n = n + 1;
        rest = Text::slice(rest, i + 1, rest.len());
    }
    return n;
}

/// Returns `s` with leading `prefix` removed when present
/// (Go's `strings.TrimPrefix`).
pub fn TrimPrefix(s: String, prefix: String) -> String {
    if Text::starts_with(s, prefix) {
        return Text::slice(s, prefix.len(), s.len());
    }
    return s;
}

/// Returns `s` with trailing `suffix` removed when present
/// (Go's `strings.TrimSuffix`).
pub fn TrimSuffix(s: String, suffix: String) -> String {
    if Text::ends_with(s, suffix) {
        return Text::slice(s, 0, s.len() - suffix.len());
    }
    return s;
}

/// Splits `s` into fields around runs of white space (Go's
/// `strings.Fields`): space, tab, newline, carriage return.
pub fn Fields(s: String) -> Dynamic {
    let out = [];
    let cur = "";
    let i = 0;
    let n = s.len();
    while i < n {
        let c = Text::slice(s, i, i + 1);
        if c == " " || c == "\t" || c == "\n" || c == "\r" {
            if cur != "" {
                out.push(cur);
                cur = "";
            }
        } else {
            cur = cur + c;
        }
        i = i + 1;
    }
    if cur != "" {
        out.push(cur);
    }
    return out;
}

/// Splits `s` around the first `sep`, returning `[before, after, found]`
/// (Go's `strings.Cut`).
pub fn Cut(s: String, sep: String) -> Dynamic {
    let i = Text::index_of(s, sep);
    if i < 0 {
        return [s, "", false];
    }
    return [Text::slice(s, 0, i), Text::slice(s, i + sep.len(), s.len()), true];
}

/// Returns `s` without `prefix` plus whether it was present (Go's
/// `strings.CutPrefix`).
pub fn CutPrefix(s: String, prefix: String) -> Dynamic {
    if Text::starts_with(s, prefix) {
        return [Text::slice(s, prefix.len(), s.len()), true];
    }
    return [s, false];
}

/// Returns `s` without `suffix` plus whether it was present (Go's
/// `strings.CutSuffix`).
pub fn CutSuffix(s: String, suffix: String) -> Dynamic {
    if Text::ends_with(s, suffix) {
        return [Text::slice(s, 0, s.len() - suffix.len()), true];
    }
    return [s, false];
}

/// Trims leading and trailing code points from `cutset` (Go's
/// `strings.Trim`).
pub fn Trim(s: String, cutset: String) -> String {
    return TrimRight(TrimLeft(s, cutset), cutset);
}

/// Trims leading code points from `cutset` (Go's `strings.TrimLeft`).
pub fn TrimLeft(s: String, cutset: String) -> String {
    let i = 0;
    while i < s.len() && Text::index_of(cutset, Text::slice(s, i, i + 1)) >= 0 {
        i = i + 1;
    }
    return Text::slice(s, i, s.len());
}

/// Trims trailing code points from `cutset` (Go's `strings.TrimRight`).
pub fn TrimRight(s: String, cutset: String) -> String {
    let j = s.len();
    while j > 0 && Text::index_of(cutset, Text::slice(s, j - 1, j)) >= 0 {
        j = j - 1;
    }
    return Text::slice(s, 0, j);
}
