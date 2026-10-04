// Lexicon Standard Library — expvar.
// Go-parity exported variables (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3).
// Named int/string publishes with JSON rendering for `/debug/vars`
// handlers. Import as `import std::expvar;`.

let expvar_names = [];
let expvar_kinds = [];
let expvar_vals = [];

fn expvar_idx(name: String) -> i64 {
    let i = 0;
    while i < expvar_names.len() {
        if expvar_names[i] == name {
            return i;
        }
        i = i + 1;
    }
    return -1;
}

/// Publishes an int variable (Go's `expvar.NewInt`).
pub fn NewInt(name: String) -> void {
    if expvar_idx(name) < 0 {
        expvar_names.push(name);
        expvar_kinds.push("int");
        expvar_vals.push(0);
    }
}

/// Publishes a string variable (Go's `expvar.NewString`).
pub fn NewString(name: String) -> void {
    if expvar_idx(name) < 0 {
        expvar_names.push(name);
        expvar_kinds.push("string");
        expvar_vals.push("");
    }
}

/// Adds `d` to an int var (Go's `Int.Add`).
pub fn Add(name: String, d: i64) -> void {
    let i = expvar_idx(name);
    if i >= 0 {
        let row = expvar_vals[i];
        expvar_vals[i] = row + d;
    }
}

/// Sets a var (Go's `Int.Set` / `String.Set`).
pub fn Set(name: String, v: Dynamic) -> void {
    let i = expvar_idx(name);
    if i >= 0 {
        expvar_vals[i] = v;
    }
}

/// Reads a var, or 0 when missing (Go's `expvar.Get`, value-adapted).
pub fn Get(name: String) -> Dynamic {
    let i = expvar_idx(name);
    if i >= 0 {
        return expvar_vals[i];
    }
    return 0;
}

/// Renders all vars as JSON (Go's `/debug/vars` body).
pub fn Render() -> String {
    let out = "{";
    let i = 0;
    while i < expvar_names.len() {
        if i > 0 {
            out = out + ",";
        }
        out = out + "\"" + expvar_names[i].to_string() + "\":";
        if expvar_kinds[i] == "string" {
            out = out + "\"" + expvar_vals[i].to_string() + "\"";
        } else {
            out = out + expvar_vals[i].to_string();
        }
        i = i + 1;
    }
    return out + "}";
}
