// Lexicon Standard Library — regexp.
// Go-parity regular expressions, documented subset
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 4). Backtracking matcher over
// the pattern text (no separate compilation step):
// literals, `.`, `\d\D\w\W\s\S`, `[...]`, `[^...]`, `* + ?`, `{n,m}`,
// `^ $`, `|` and `( )` groups. Groups are non-capturing for `Find`
// (whole match only — `FindSubmatch` is Fase 10 material).
// Import as `import std::regexp;`.

/// Whole/partial match result with the end offset.
pub struct MatchAt {
    ok: bool,
    end: i64,
}

/// A located match (Go's `FindString` triple, struct-adapted).
pub struct FindResult {
    text: String,
    start: i64,
    end: i64,
    ok: bool,
}

/// Reports whether `pattern` matches any substring of `s`
/// (Go's `regexp.MatchString`).
pub fn Match(pattern: String, s: String) -> bool {
    return Find(pattern, s).ok;
}

/// Returns the leftmost match (Go's `FindString`, location-adapted).
pub fn Find(pattern: String, s: String) -> FindResult {
    let si = 0;
    while si <= s.len() {
        let r = match_here(pattern, 0, s, si);
        if r.ok {
            return FindResult { text: Text::slice(s, si, r.end), start: si, end: r.end, ok: true };
        }
        si = si + 1;
    }
    return FindResult { text: "", start: -1, end: -1, ok: false };
}

/// Returns every non-overlapping match (Go's `FindAllString`, `-1` form).
pub fn FindAll(pattern: String, s: String) -> Dynamic {
    let out = [];
    let si = 0;
    while si <= s.len() {
        let r = match_here(pattern, 0, s, si);
        if r.ok {
            out.push(Text::slice(s, si, r.end));
            if r.end > si {
                si = r.end;
            } else {
                si = si + 1;
            }
        } else {
            si = si + 1;
        }
    }
    return out;
}

/// Replaces every match with `repl` (Go's `ReplaceAllString`, literal
/// replacement — no `$1` expansion yet).
pub fn ReplaceAll(s: String, pattern: String, repl: String) -> Dynamic {
    let out = "";
    let si = 0;
    while si <= s.len() {
        let r = match_here(pattern, 0, s, si);
        if r.ok {
            out = out + repl;
            if r.end > si {
                si = r.end;
            } else {
                out = out + Text::slice(s, si, si + 1);
                si = si + 1;
            }
        } else {
            if si < s.len() {
                out = out + Text::slice(s, si, si + 1);
            }
            si = si + 1;
        }
    }
    return out;
}

/// Splits around each match (Go's `Split`, empty-match guarded).
pub fn Split(s: String, pattern: String) -> Dynamic {
    let out = [];
    let cur = "";
    let si = 0;
    while si <= s.len() {
        let r = match_here(pattern, 0, s, si);
        if r.ok && r.end > si {
            out.push(cur);
            cur = "";
            si = r.end;
        } else {
            if si < s.len() {
                cur = cur + Text::slice(s, si, si + 1);
            }
            si = si + 1;
        }
    }
    out.push(cur);
    return out;
}

/// Escapes all meta characters so the result matches `s` literally
/// (Go's `regexp.QuoteMeta`).
pub fn QuoteMeta(s: String) -> String {
    let out = "";
    let i = 0;
    while i < s.len() {
        let c = Text::slice(s, i, i + 1);
        if c == "\\" || c == "." || c == "+" || c == "*" || c == "?" || c == "(" || c == ")" || c == "|" || c == "[" || c == "]" || c == "{" || c == "}" || c == "^" || c == "$" {
            out = out + "\\";
        }
        out = out + c;
        i = i + 1;
    }
    return out;
}

// ---------------------------------------------------------------------------
// internals
// ---------------------------------------------------------------------------

fn match_here(pat: String, pi: i64, s: String, si: i64) -> MatchAt {
    let fail = MatchAt { ok: false, end: -1 };
    if pi >= pat.len() {
        return MatchAt { ok: true, end: si };
    }
    let c = Text::slice(pat, pi, pi + 1);
    if c == "^" {
        if si != 0 {
            return fail;
        }
        return match_here(pat, pi + 1, s, si);
    }
    if c == "$" && pi + 1 >= pat.len() {
        if si == s.len() {
            return MatchAt { ok: true, end: si };
        }
        return fail;
    }
    let alt = top_pipe(pat, pi);
    if alt >= 0 {
        let left = Text::slice(pat, pi, alt);
        let right = Text::slice(pat, alt + 1, pat.len());
        let a = match_here(left, 0, s, si);
        if a.ok {
            return a;
        }
        return match_here(right, 0, s, si);
    }
    if c == "(" {
        let close = find_close(pat, pi);
        if close < 0 {
            return fail;
        }
        let inner = Text::slice(pat, pi + 1, close);
        let rest = close + 1;
        let q = quant(pat, rest);
        let after = q[2];
        let lo = q[0];
        let hi = q[1];
        if hi < 0 || hi > s.len() - si + 1 {
            hi = s.len() - si + 1;
        }
        let count = hi;
        while count >= lo {
            let g = group_times(inner, count, s, si);
            if g.ok {
                let tail = match_here(pat, after, s, g.end);
                if tail.ok {
                    return tail;
                }
            }
            count = count - 1;
        }
        return fail;
    }
    if c == ")" {
        return fail;
    }
    let a = parse_atom(pat, pi);
    let q2 = quant(pat, a[4]);
    let after2 = q2[2];
    let lo2 = q2[0];
    let hi2 = q2[1];
    if hi2 < 0 || hi2 > s.len() - si {
        hi2 = s.len() - si;
    }
    let n = hi2;
    while n >= lo2 {
        if atom_times(a, n, s, si) {
            let tail2 = match_here(pat, after2, s, si + n);
            if tail2.ok {
                return tail2;
            }
        }
        n = n - 1;
    }
    return fail;
}

fn group_times(inner: String, count: i64, s: String, si: i64) -> MatchAt {
    let fail = MatchAt { ok: false, end: -1 };
    let pos = si;
    let k = 0;
    while k < count {
        let r = match_here(inner, 0, s, pos);
        if !r.ok {
            return fail;
        }
        if r.end <= pos {
            return MatchAt { ok: true, end: pos };
        }
        pos = r.end;
        k = k + 1;
    }
    return MatchAt { ok: true, end: pos };
}

fn atom_times(a: Dynamic, n: i64, s: String, si: i64) -> bool {
    let k = 0;
    while k < n {
        if si + k >= s.len() {
            return false;
        }
        if !atom_one(a, Text::slice(s, si + k, si + k + 1)) {
            return false;
        }
        k = k + 1;
    }
    return true;
}

// Atom descriptor: [kind, s1, i1, i2, next_pi].
// kind: lit | dot | digit | word | space | class.
fn parse_atom(pat: String, pi: i64) -> Dynamic {
    let c = Text::slice(pat, pi, pi + 1);
    if c == "\\" && pi + 1 < pat.len() {
        let e = Text::slice(pat, pi + 1, pi + 2);
        if e == "d" {
            return ["digit", "", 0, 0, pi + 2];
        }
        if e == "D" {
            return ["digit", "", 1, 0, pi + 2];
        }
        if e == "w" {
            return ["word", "", 0, 0, pi + 2];
        }
        if e == "W" {
            return ["word", "", 1, 0, pi + 2];
        }
        if e == "s" {
            return ["space", "", 0, 0, pi + 2];
        }
        if e == "S" {
            return ["space", "", 1, 0, pi + 2];
        }
        return ["lit", e, 0, 0, pi + 2];
    }
    if c == "." {
        return ["dot", "", 0, 0, pi + 1];
    }
    if c == "[" {
        let j = pi + 1;
        let neg = 0;
        if j < pat.len() && Text::slice(pat, j, j + 1) == "^" {
            neg = 1;
            j = j + 1;
        }
        if j < pat.len() && Text::slice(pat, j, j + 1) == "]" {
            j = j + 1;
        }
        while j < pat.len() && Text::slice(pat, j, j + 1) != "]" {
            if Text::slice(pat, j, j + 1) == "\\" {
                j = j + 1;
            }
            j = j + 1;
        }
        let body = Text::slice(pat, pi + 1, j);
        return ["class", body, neg, 0, j + 1];
    }
    return ["lit", c, 0, 0, pi + 1];
}

fn atom_one(a: Dynamic, c: String) -> bool {
    let kind = a[0];
    if kind == "dot" {
        return c != "\n";
    }
    if kind == "lit" {
        return c == a[1];
    }
    if kind == "digit" {
        let d = c >= "0" && c <= "9";
        if a[2] == 1 {
            return !d;
        }
        return d;
    }
    if kind == "word" {
        let w = (c >= "a" && c <= "z") || (c >= "A" && c <= "Z") || (c >= "0" && c <= "9") || c == "_";
        if a[2] == 1 {
            return !w;
        }
        return w;
    }
    if kind == "space" {
        let sp = c == " " || c == "\t" || c == "\n" || c == "\r";
        if a[2] == 1 {
            return !sp;
        }
        return sp;
    }
    if kind == "class" {
        return class_hit(a[1], c, a[2]);
    }
    return false;
}

fn class_hit(body: String, c: String, neg: i64) -> bool {
    let hit = false;
    let k = 0;
    let bb = body;
    if neg == 1 && bb.len() > 0 && Text::slice(bb, 0, 1) == "^" {
        bb = Text::slice(bb, 1, bb.len());
    }
    while k < bb.len() {
        let ch = Text::slice(bb, k, k + 1);
        if ch == "\\" && k + 1 < bb.len() {
            let e = Text::slice(bb, k + 1, k + 2);
            if e == "d" && c >= "0" && c <= "9" {
                hit = true;
            }
            if e == "w" && ((c >= "a" && c <= "z") || (c >= "A" && c <= "Z") || (c >= "0" && c <= "9") || c == "_") {
                hit = true;
            }
            if e == "s" && (c == " " || c == "\t" || c == "\n" || c == "\r") {
                hit = true;
            }
            if e != "d" && e != "w" && e != "s" && c == e {
                hit = true;
            }
            k = k + 2;
            continue;
        }
        if k + 2 < bb.len() && Text::slice(bb, k + 1, k + 2) == "-" && Text::slice(bb, k + 2, k + 3) != "]" {
            let lo = ch;
            let hi = Text::slice(bb, k + 2, k + 3);
            if lo <= c && c <= hi {
                hit = true;
            }
            k = k + 3;
            continue;
        }
        if ch == c {
            hit = true;
        }
        k = k + 1;
    }
    if neg == 1 {
        return !hit;
    }
    return hit;
}

// Quantifier at `pi`: [min, max, next_pi], max -1 = unbounded.
fn quant(pat: String, pi: i64) -> Dynamic {
    if pi >= pat.len() {
        return [1, 1, pi];
    }
    let c = Text::slice(pat, pi, pi + 1);
    if c == "*" {
        return [0, -1, pi + 1];
    }
    if c == "+" {
        return [1, -1, pi + 1];
    }
    if c == "?" {
        return [0, 1, pi + 1];
    }
    if c == "{" {
        let j = pi + 1;
        let ns = "";
        while j < pat.len() && Text::slice(pat, j, j + 1) >= "0" && Text::slice(pat, j, j + 1) <= "9" {
            ns = ns + Text::slice(pat, j, j + 1);
            j = j + 1;
        }
        if ns == "" {
            return [1, 1, pi];
        }
        let lo = 0 + atoi(ns);
        let hi = lo;
        if j < pat.len() && Text::slice(pat, j, j + 1) == "," {
            j = j + 1;
            let ms = "";
            while j < pat.len() && Text::slice(pat, j, j + 1) >= "0" && Text::slice(pat, j, j + 1) <= "9" {
                ms = ms + Text::slice(pat, j, j + 1);
                j = j + 1;
            }
            if ms == "" {
                hi = -1;
            } else {
                hi = 0 + atoi(ms);
            }
        }
        if j < pat.len() && Text::slice(pat, j, j + 1) == "}" {
            return [lo, hi, j + 1];
        }
        return [1, 1, pi];
    }
    return [1, 1, pi];
}

fn atoi(s: String) -> i64 {
    let v = 0;
    let i = 0;
    while i < s.len() {
        let d = Text::index_of("0123456789", Text::slice(s, i, i + 1));
        v = v * 10 + d;
        i = i + 1;
    }
    return v;
}

// Index of the `|` splitting this level, or -1.
fn top_pipe(pat: String, pi: i64) -> i64 {
    let depth = 0;
    let j = pi;
    while j < pat.len() {
        let c = Text::slice(pat, j, j + 1);
        if c == "\\" {
            j = j + 2;
            continue;
        }
        if c == "[" {
            j = j + 1;
            while j < pat.len() && Text::slice(pat, j, j + 1) != "]" {
                if Text::slice(pat, j, j + 1) == "\\" {
                    j = j + 1;
                }
                j = j + 1;
            }
            j = j + 1;
            continue;
        }
        if c == "(" {
            depth = depth + 1;
        }
        if c == ")" {
            depth = depth - 1;
        }
        if c == "|" && depth == 0 {
            return j;
        }
        j = j + 1;
    }
    return -1;
}

// Index of the `)` closing the group opened at `pi`, or -1.
fn find_close(pat: String, pi: i64) -> i64 {
    let depth = 0;
    let j = pi;
    while j < pat.len() {
        let c = Text::slice(pat, j, j + 1);
        if c == "\\" {
            j = j + 2;
            continue;
        }
        if c == "[" {
            j = j + 1;
            while j < pat.len() && Text::slice(pat, j, j + 1) != "]" {
                if Text::slice(pat, j, j + 1) == "\\" {
                    j = j + 1;
                }
                j = j + 1;
            }
            j = j + 1;
            continue;
        }
        if c == "(" {
            depth = depth + 1;
        }
        if c == ")" {
            depth = depth - 1;
            if depth == 0 {
                return j;
            }
        }
        j = j + 1;
    }
    return -1;
}
