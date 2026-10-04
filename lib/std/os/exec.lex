// Lexicon Standard Library — os/exec.
// Go-parity external commands (plan: Docs/PLANO_GO_FULL_PARITY.md, Fase 3).
// Execution delegates to the native `Process::output` builtin (spawn +
// capture); this layer keeps Go's `Cmd` shapes. Import as
// `import std::os::exec;`.

/// A prepared command (Go's `exec.Cmd`).
pub struct Cmd {
    prog: String,
    argv: Dynamic,
    input: String,
}

/// Captured result (Go's `(output, err)` pair, struct-adapted).
pub struct OutputResult {
    out: String,
    err: String,
    code: i64,
    ok: bool,
}

/// Prepares `prog` with `argv` (Go's `exec.Command`).
pub fn Command(prog: String, argv: Dynamic) -> Cmd {
    return Cmd { prog: prog, argv: argv, input: "" };
}

/// Attaches stdin bytes (Go's `cmd.Stdin` assignment, value-adapted).
pub fn WithInput(c: Cmd, input: String) -> Cmd {
    return Cmd { prog: c.prog, argv: c.argv, input: input };
}

/// Runs the command and captures stdout (Go's `cmd.Output`).
pub fn Output(c: Cmd) -> OutputResult {
    let o = Process::output(c.prog, c.argv, c.input);
    return OutputResult { out: o.stdout, err: o.stderr, code: o.code, ok: o.code == 0 };
}

/// Runs and captures stdout+stderr interleaved-ish (Go's
/// `cmd.CombinedOutput`: stderr appended after stdout here, documented).
pub fn CombinedOutput(c: Cmd) -> OutputResult {
    let r = Output(c);
    return OutputResult { out: r.out + r.err, err: "", code: r.code, ok: r.ok };
}

/// Runs and reports success only (Go's `cmd.Run`).
pub fn Run(c: Cmd) -> bool {
    return Output(c).ok;
}

/// Looks for `prog` on PATH (Go's `exec.LookPath`): absolute/relative
/// paths pass through when they exist; bare names resolve via `PATH`.
pub fn LookPath(prog: String) -> String {
    if Text::index_of(prog, "/") >= 0 || Text::index_of(prog, "\\") >= 0 {
        if File::exists(prog) {
            return prog;
        }
        return "";
    }
    let path = Env::get("PATH");
    if path == "NOT_FOUND" {
        return "";
    }
    let sep = ":";
    if Env::get("OS") == "Windows_NT" {
        sep = ";";
    }
    let dirs = path.split(sep);
    let i = 0;
    while i < dirs.len() {
        let cand = dirs[i] + "/" + prog;
        if File::exists(cand) {
            return cand;
        }
        if Env::get("OS") == "Windows_NT" && File::exists(cand + ".exe") {
            return cand + ".exe";
        }
        i = i + 1;
    }
    return "";
}
