// Lexicon Standard Library — path.
// Go-parity slash-separated path manipulation
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 1). Forward slashes only,
// exactly like Go's `path` package (OS paths with backslashes live in
// path/filepath, Fase 3).

import std::strings;

/// Reports whether the path is absolute (begins with '/').
pub fn IsAbs(p: String) -> bool {
    return strings::HasPrefix(p, "/");
}

/// Returns the last element of the path (Go's `path.Base`):
/// strips trailing slashes; ".." for ".."; "." for "" or all-slash.
pub fn Base(p: String) -> String {
    if p == "" {
        return ".";
    }
    // Strip trailing slashes.
    let n = p.len();
    while n > 0 && (p.char_at(n - 1) as String) == "/" {
        n = n - 1;
    }
    if n == 0 {
        return "/";
    }
    // Find start of the last segment.
    let i = n - 1;
    while i >= 0 && (p.char_at(i) as String) != "/" {
        i = i - 1;
    }
    return strings::Slice(p, i + 1, n);
}

/// Returns all but the last element of the path (Go's `path.Dir`).
pub fn Dir(p: String) -> String {
    let n = p.len();
    // Trim trailing slashes (unless the whole path is one slash run).
    let rooted = strings::HasPrefix(p, "/");
    let i = n;
    while i > 0 && (p.char_at(i - 1) as String) == "/" {
        i = i - 1;
    }
    if i == 0 {
        if rooted {
            return "/";
        }
        return ".";
    }
    // Trim the last segment.
    let j = i;
    while j > 0 && (p.char_at(j - 1) as String) != "/" {
        j = j - 1;
    }
    if j == 0 {
        if rooted {
            return "/";
        }
        return ".";
    }
    // Trim trailing slashes of the dir part.
    let k = j;
    while k > 1 && (p.char_at(k - 1) as String) == "/" {
        k = k - 1;
    }
    return strings::Slice(p, 0, k);
}

/// Returns the file name extension (Go's `path.Ext`): the suffix from
/// the final dot of the final element; "" when absent.
pub fn Ext(p: String) -> String {
    let i = p.len() - 1;
    while i >= 0 {
        let c = p.char_at(i) as String;
        if c == "/" {
            return "";
        }
        if c == "." {
            return strings::Slice(p, i, p.len());
        }
        i = i - 1;
    }
    return "";
}

/// Joins any number of path elements with slashes, dropping empty
/// elements; result is Cleaned (Go's `path.Join`, list-adapted).
pub fn Join(elems: Dynamic) -> String {
    let joined = "";
    let i = 0;
    let n = elems.len();
    while i < n {
        let e = elems[i].to_string();
        if e != "" {
            if joined == "" {
                joined = e;
            } else {
                joined = joined + "/" + e;
            }
        }
        i = i + 1;
    }
    return Clean(joined);
}

/// Splits `p` into directory and file name (Go's `path.Split`).
/// Returns a two-element list [dir, file].
pub fn Split(p: String) -> Dynamic {
    let i = p.len() - 1;
    while i >= 0 {
        if (p.char_at(i) as String) == "/" {
            return [strings::Slice(p, 0, i + 1), strings::Slice(p, i + 1, p.len())];
        }
        i = i - 1;
    }
    return ["", p];
}

/// Lexically cleans `p` (Go's `path.Clean`): resolves "." and "..",
/// collapses slash runs, preserves a leading slash.
pub fn Clean(p: String) -> String {
    if p == "" {
        return ".";
    }
    let rooted = strings::HasPrefix(p, "/");
    let parts = p.split("/");
    let out = [];
    let i = 0;
    let n = parts.len();
    while i < n {
        let seg = parts[i];
        if seg == "" || seg == "." {
            i = i + 1;
            continue;
        }
        if seg == ".." {
            if out.len() > 0 && out[out.len() - 1] != ".." {
                out.pop();
            } else {
                if !rooted {
                    out.push("..");
                }
            }
            i = i + 1;
            continue;
        }
        out.push(seg);
        i = i + 1;
    }
    let result = "";
    if rooted {
        result = "/";
    }
    let k = 0;
    let m = out.len();
    while k < m {
        if k > 0 {
            result = result + "/";
        }
        result = result + out[k];
        k = k + 1;
    }
    if result == "" {
        if rooted {
            return "/";
        }
        return ".";
    }
    return result;
}
