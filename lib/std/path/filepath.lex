// Lexicon Standard Library — path/filepath.
// Go-parity OS-path manipulation (plan: Docs/PLANO_GO_FULL_PARITY.md,
// Fase 3). Unlike `path` (slashes only), the separator follows the host:
// `\` when `OS=Windows_NT`, `/` elsewhere — like Go's `filepath`.
// Import as `import std::path::filepath;`.

import std::strings;

/// Returns the OS separator as a string (Go's `filepath.Separator`).
pub fn Separator() -> String {
    if Env::get("OS") == "Windows_NT" {
        return "\\";
    }
    return "/";
}

/// Reports whether `sep` is a path separator on this host.
pub fn IsSeparator(c: String) -> bool {
    if c == "/" {
        return true;
    }
    return Separator() == "\\" && c == "\\";
}

fn fp_to_slash(p: String) -> String {
    if Separator() == "\\" {
        return p.replace_all("\\", "/");
    }
    return p;
}

fn fp_from_slash(p: String) -> String {
    if Separator() == "\\" {
        return p.replace_all("/", "\\");
    }
    return p;
}

/// Reports whether the path is absolute (Go's `filepath.IsAbs`).
pub fn IsAbs(p: String) -> bool {
    let s = fp_to_slash(p);
    if strings::HasPrefix(s, "/") {
        return true;
    }
    if s.len() >= 3 && Text::slice(s, 1, 3) == ":/" {
        return true;
    }
    if s.len() >= 2 && Text::slice(s, 1, 2) == ":" && s.len() == 2 {
        return true;
    }
    return false;
}

/// Joins elements with the OS separator and cleans the result
/// (Go's `filepath.Join`, list-adapted).
pub fn Join(elems: Dynamic) -> String {
    let joined = "";
    let i = 0;
    while i < elems.len() {
        let e = elems[i].to_string();
        if e != "" {
            if joined == "" {
                joined = e;
            } else {
                joined = joined + Separator() + e;
            }
        }
        i = i + 1;
    }
    return Clean(joined);
}

/// Splits `p` into directory and file (Go's `filepath.Split`).
pub fn Split(p: String) -> Dynamic {
    let s = fp_to_slash(p);
    let i = s.len() - 1;
    while i >= 0 {
        if Text::slice(s, i, i + 1) == "/" {
            let dir = fp_from_slash(Text::slice(s, 0, i + 1));
            let file = Text::slice(s, i + 1, s.len());
            return [dir, file];
        }
        i = i - 1;
    }
    return ["", p];
}

/// Returns the last element (Go's `filepath.Base`).
pub fn Base(p: String) -> String {
    // Qualified self-call: the flattened namespace would otherwise
    // resolve `Split` to `strings::Split` (loaded first as our import).
    let sp = filepath::Split(p);
    let file = sp[1];
    if file == "" {
        return ".";
    }
    return file;
}

/// Returns all but the last element (Go's `filepath.Dir`).
pub fn Dir(p: String) -> String {
    let sp = filepath::Split(p);
    let dir = sp[0];
    if dir == "" {
        return ".";
    }
    if dir.len() > 1 && Text::slice(dir, dir.len() - 1, dir.len()) == Separator() {
        return Text::slice(dir, 0, dir.len() - 1);
    }
    return dir;
}

/// Returns the extension (Go's `filepath.Ext`).
pub fn Ext(p: String) -> String {
    let b = Base(p);
    let i = Text::last_index_of(b, ".");
    if i <= 0 {
        return "";
    }
    return Text::slice(b, i, b.len());
}

/// Cleans the path (Go's `filepath.Clean`): slash-normalized, `.`/`..`
/// resolved, OS separators restored.
pub fn Clean(p: String) -> String {
    if p == "" {
        return ".";
    }
    let s = fp_to_slash(p);
    let rooted = strings::HasPrefix(s, "/");
    let parts = s.split("/");
    let out = [];
    let i = 0;
    while i < parts.len() {
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
    while k < out.len() {
        if k > 0 {
            result = result + "/";
        }
        result = result + out[k];
        k = k + 1;
    }
    if result == "" {
        if rooted {
            return fp_from_slash("/");
        }
        return ".";
    }
    return fp_from_slash(result);
}

/// Lists every file under `dir`, recursively (Go's `filepath.Walk`
/// result-adapted: returns the path list instead of driving a callback).
pub fn Walk(dir: String) -> Dynamic {
    let out = [dir];
    let i = 0;
    while i < out.len() {
        let cur = out[i];
        if File::is_dir(cur) {
            let names = File::read_dir(cur);
            let j = 0;
            while j < names.len() {
                let sep = "/";
                if !strings::HasSuffix(cur, "/") && !strings::HasSuffix(cur, "\\") {
                    sep = Separator();
                } else {
                    sep = "";
                }
                out.push(cur + sep + names[j]);
                j = j + 1;
            }
        }
        i = i + 1;
    }
    return out;
}

/// Matches `name` against `pattern` with `*`/`?` (Go's `path.Match`
/// subset used by `filepath.Glob`, `*` never crosses separators here —
/// single-level names, like Go).
pub fn Match(pattern: String, name: String) -> bool {
    return fp_match(pattern, 0, name, 0);
}

fn fp_match(pat: String, pi: i64, s: String, si: i64) -> bool {
    let pn = pat.len();
    let sn = s.len();
    while pi < pn {
        let pc = Text::slice(pat, pi, pi + 1);
        if pc == "*" {
            let k = si;
            while k <= sn {
                if fp_match(pat, pi + 1, s, k) {
                    return true;
                }
                k = k + 1;
            }
            return false;
        }
        if si >= sn {
            return false;
        }
        let sc = Text::slice(s, si, si + 1);
        if pc == "?" {
            pi = pi + 1;
            si = si + 1;
        } else {
            if pc == "\\" {
                pi = pi + 1;
                if pi >= pn {
                    return false;
                }
                pc = Text::slice(pat, pi, pi + 1);
            }
            if pc != sc {
                return false;
            }
            pi = pi + 1;
            si = si + 1;
        }
    }
    return si >= sn;
}
