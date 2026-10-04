// Lexicon Standard Library — flag.
// Go-parity command-line flags (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3).
// Definitions accumulate in module globals; `Parse` reads `os.Args()`
// (`--name=value`, `--name value`, `-name value`, boolean `--flag`).
// Import as `import std::flag;`.

let flag_defs = [];
let flag_parsed = false;

/// Declares a string flag (Go's `flag.String`).
pub fn String(name: String, def: String, usage: String) -> void {
    flag_defs.push(["string", name, def, usage]);
    flag_parsed = false;
}

/// Declares an integer flag (Go's `flag.Int`).
pub fn Int(name: String, def: i64, usage: String) -> void {
    flag_defs.push(["int", name, def.to_string(), usage]);
    flag_parsed = false;
}

/// Declares a boolean flag (Go's `flag.Bool`).
pub fn Bool(name: String, def: bool, usage: String) -> void {
    flag_defs.push(["bool", name, def.to_string(), usage]);
    flag_parsed = false;
}

fn find_def(name: String) -> i64 {
    let i = 0;
    while i < flag_defs.len() {
        if flag_defs[i][1] == name {
            return i;
        }
        i = i + 1;
    }
    return -1;
}

fn set_val(di: i64, raw: String) -> void {
    // Nested `flag_defs[di][2] = raw` is not assignable (index target must
    // be a plain identifier), so round-trip through a row local.
    let row = flag_defs[di];
    row[2] = raw;
    flag_defs[di] = row;
}

/// Parses `Process::args()` into the declared flags (Go's `flag.Parse`).
/// Unknown `--flags` abort with a usage line (Go behavior).
pub fn Parse() -> void {
    let argv = Process::args();
    let vals = [];
    let i = 0;
    while i < argv.len() {
        vals.push(argv[i]);
        i = i + 1;
    }
    let k = 1;
    while k < vals.len() {
        let a = vals[k].to_string();
        if Text::starts_with(a, "--") || (Text::starts_with(a, "-") && a.len() > 1) {
            let name = a;
            if Text::starts_with(a, "--") {
                name = Text::slice(a, 2, a.len());
            } else {
                name = Text::slice(a, 1, a.len());
            }
            let v = "";
            let has_eq = Text::index_of(name, "=") >= 0;
            if has_eq {
                let eq = Text::index_of(name, "=");
                v = Text::slice(name, eq + 1, name.len());
                name = Text::slice(name, 0, eq);
            }
            let di = find_def(name);
            if di < 0 {
                Console::log("flag: unknown flag -" + name);
                Usage();
                panic("flag: unknown flag -" + name);
            }
            let kind = flag_defs[di][0];
            if kind == "bool" && !has_eq {
                if k + 1 < vals.len() && vals[k + 1].to_string() != "true" && vals[k + 1].to_string() != "false" {
                    set_val(di, "true");
                } else {
                    if k + 1 < vals.len() {
                        set_val(di, vals[k + 1].to_string());
                        k = k + 1;
                    } else {
                        set_val(di, "true");
                    }
                }
            } else {
                if !has_eq {
                    if k + 1 < vals.len() {
                        v = vals[k + 1].to_string();
                        k = k + 1;
                    }
                }
                set_val(di, v);
            }
        }
        k = k + 1;
    }
    flag_parsed = true;
}

/// Returns the string value (Go's `*flag.String(...)` dereference).
pub fn GetString(name: String) -> String {
    let di = find_def(name);
    if di < 0 {
        return "";
    }
    return flag_defs[di][2].to_string();
}

/// Returns the integer value (invalid → 0, like Go's failed `Atoi` use).
pub fn GetInt(name: String) -> i64 {
    let raw = GetString(name);
    let neg = false;
    let s = raw;
    if raw.len() > 0 && Text::slice(raw, 0, 1) == "-" {
        neg = true;
        s = Text::slice(raw, 1, raw.len());
    }
    let v = 0;
    let i = 0;
    while i < s.len() {
        let d = Text::index_of("0123456789", Text::slice(s, i, i + 1));
        if d < 0 {
            return 0;
        }
        v = v * 10 + d;
        i = i + 1;
    }
    if neg {
        return 0 - v;
    }
    return v;
}

/// Returns the boolean value.
pub fn GetBool(name: String) -> bool {
    return GetString(name) == "true";
}

/// Prints usage (Go's `flag.Usage`).
pub fn Usage() -> void {
    Console::log("Usage:");
    let i = 0;
    while i < flag_defs.len() {
        Console::log("  -" + flag_defs[i][1].to_string() + " (" + flag_defs[i][0].to_string() + ") " + flag_defs[i][3].to_string());
        i = i + 1;
    }
}
