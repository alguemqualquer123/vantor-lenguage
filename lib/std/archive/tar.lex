// Lexicon Standard Library — archive/tar.
// Go-parity ustar archives, uncompressed
// (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 7). Bytes travel as strings
// (code points 0-255 round-trip exactly). Value semantics: readers
// advance by return — `let n = tar::Next(r); r = n.reader;`.
// Import as `import std::archive::tar;`.

/// Entry metadata (Go's `tar.Header`, portable subset).
pub struct Header {
    name: String,
    size: i64,
    mode: i64,
    mtime: i64,
    dir: bool,
}

/// A reader over an archive string (Go's `tar.Reader`).
pub struct Reader {
    data: String,
    pos: i64,
}

/// One step: advanced `reader`, entry `header` + `data`, `ok`.
pub struct NextResult {
    reader: Reader,
    header: Header,
    data: String,
    ok: bool,
}

/// Builds a reader (Go's `tar.NewReader`).
pub fn NewReader(data: String) -> Reader {
    return Reader { data: data, pos: 0 };
}

/// Reads the next entry (Go's `tr.Next` + `io.ReadAll` combined: headers
/// and data arrive together, no streaming reads in the tree-walker).
pub fn Next(r: Reader) -> NextResult {
    let done = NextResult { reader: r, header: Header { name: "", size: 0, mode: 0, mtime: 0, dir: false }, data: "", ok: false };
    if r.pos + 512 > r.data.len() {
        return done;
    }
    let head = Text::slice(r.data, r.pos, r.pos + 512);
    if is_zero(head) {
        return done;
    }
    let name = cstr(Text::slice(head, 0, 100));
    let size = oct(Text::slice(head, 124, 136));
    let mode = oct(Text::slice(head, 100, 108));
    let mtime = oct(Text::slice(head, 136, 148));
    let flag = Text::slice(head, 156, 157);
    if size < 0 {
        return done;
    }
    let start = r.pos + 512;
    let end = start + size;
    if end > r.data.len() {
        return done;
    }
    let pad = (512 - size % 512) % 512;
    let next = Reader { data: r.data, pos: end + pad };
    let h = Header { name: name, size: size, mode: mode, mtime: mtime, dir: flag == "5" };
    return NextResult { reader: next, header: h, data: Text::slice(r.data, start, end), ok: true };
}

/// Appends one file entry to an archive string (Go's `tar.Writer`
/// `WriteHeader` + `Write`, document-adapted: returns the new archive).
pub fn AppendFile(archive: String, name: String, data: String, mode: i64) -> String {
    return archive + header_of(name, data.len(), mode, "0") + pad_to(data, 512);
}

/// Appends one directory entry.
pub fn AppendDir(archive: String, name: String, mode: i64) -> String {
    let n = name;
    if !Text::ends_with(n, "/") {
        n = n + "/";
    }
    return archive + header_of(n, 0, mode, "5");
}

fn header_of(name: String, size: i64, mode: i64, flag: String) -> String {
    let h = rpad(name, 100) + octf(mode, 8) + octf(0, 8) + octf(0, 8) + octf(size, 12) + octf(0, 12) + "        " + flag;
    h = rpad(h, 257) + "ustar" + Text::from_code(0) + "00";
    h = rpad(h, 512);
    let sum = 0;
    let i = 0;
    while i < 512 {
        sum = sum + Text::code_at(h, i);
        i = i + 1;
    }
    let cs = octf(sum, 7) + Text::from_code(0);
    return Text::slice(h, 0, 148) + cs + Text::slice(h, 156, 512);
}

fn is_zero(s: String) -> bool {
    let i = 0;
    while i < s.len() {
        if Text::code_at(s, i) != 0 {
            return false;
        }
        i = i + 1;
    }
    return true;
}

fn cstr(s: String) -> String {
    let z = Text::index_of(s, Text::from_code(0));
    if z < 0 {
        return s;
    }
    return Text::slice(s, 0, z);
}

fn oct(s: String) -> i64 {
    let z = Text::index_of(s, Text::from_code(0));
    if z >= 0 {
        s = Text::slice(s, 0, z);
    }
    let t = s.trim();
    if t == "" {
        return 0;
    }
    let v = 0;
    let i = 0;
    while i < t.len() {
        let d = Text::index_of("01234567", Text::slice(t, i, i + 1));
        if d < 0 {
            return -1;
        }
        v = v * 8 + d;
        i = i + 1;
    }
    return v;
}

fn octf(v: i64, width: i64) -> String {
    let digits = "01234567";
    let out = "";
    let x = v;
    if x == 0 {
        out = "0";
    }
    while x > 0 {
        out = Text::slice(digits, x % 8, x % 8 + 1) + out;
        x = x / 8;
    }
    while out.len() < width - 1 {
        out = "0" + out;
    }
    return out + Text::from_code(0);
}

fn rpad(s: String, n: i64) -> String {
    let out = s;
    while out.len() < n {
        out = out + Text::from_code(0);
    }
    return Text::slice(out, 0, n);
}

fn pad_to(s: String, block: i64) -> String {
    let out = s;
    while out.len() % block != 0 {
        out = out + Text::from_code(0);
    }
    return out;
}
