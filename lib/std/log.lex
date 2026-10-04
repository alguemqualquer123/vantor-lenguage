// Lexicon Standard Library — log.
// Go-parity default logger (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3).
// Prefix/flags live in module globals so `SetPrefix`/`SetFlags` behave
// like Go's package-level mutators. Import as `import std::log;`.

let log_prefix = "";
let log_flags = 0;

/// Sets the default prefix (Go's `log.SetPrefix`).
pub fn SetPrefix(p: String) -> void {
    log_prefix = p;
}

/// Sets the flag bits (Go's `log.SetFlags`; bit 1 adds a UTC timestamp).
pub fn SetFlags(f: i64) -> void {
    log_flags = f;
}

/// Returns the default prefix (Go's `log.Prefix`).
pub fn Prefix() -> String {
    return log_prefix;
}

/// Returns the flag bits (Go's `log.Flags`).
pub fn Flags() -> i64 {
    return log_flags;
}

fn log_line(args: Dynamic) -> String {
    let body = "";
    let i = 0;
    while i < args.len() {
        if i > 0 {
            body = body + " ";
        }
        body = body + args[i].to_string();
        i = i + 1;
    }
    if log_flags == 1 {
        return log_prefix + Time::now_unix_ms().to_string() + " " + body;
    }
    return log_prefix + body;
}

/// Writes one line (Go's `log.Print`).
pub fn Print(args: Dynamic) -> void {
    Console::log(log_line(args));
}

/// Writes one line (Go's `log.Println`, space-joined + newline).
pub fn Println(args: Dynamic) -> void {
    Console::log(log_line(args));
}

/// Writes a formatted line (Go's `log.Printf`; `%v/%d/%s/%f/%t/%x/%q`).
pub fn Printf(format: String, args: Dynamic) -> void {
    Console::log(log_prefix + log_sprintf(format, args));
}

fn log_sprintf(format: String, args: Dynamic) -> String {
    let out = "";
    let i = 0;
    let n = format.len();
    let argi = 0;
    while i < n {
        let c = Text::slice(format, i, i + 1);
        if c != "%" {
            out = out + c;
            i = i + 1;
        } else {
            i = i + 1;
            if i >= n {
                out = out + "%!(NOVERB)";
                break;
            }
            let verb = Text::slice(format, i, i + 1);
            i = i + 1;
            if verb == "%" {
                out = out + "%";
            } else {
                if argi >= args.len() {
                    out = out + "%!" + verb + "(MISSING)";
                } else {
                    let a = args[argi];
                    argi = argi + 1;
                    out = out + a.to_string();
                }
            }
        }
    }
    return out;
}

/// Writes one line and aborts with code 1 (Go's `log.Fatal`).
pub fn Fatal(args: Dynamic) -> void {
    Console::log(log_line(args));
    Process::exit(1);
}

/// Aborts with the message (Go's `log.Panic`).
pub fn Panic(args: Dynamic) -> void {
    panic(log_line(args));
}
