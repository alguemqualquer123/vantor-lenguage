// Lexicon Standard Library — os.
// Go-parity operating-system surface (plan: Docs/PLANO_GO_FULL_PARITY.md,
// Fase 3). Every call delegates to a native builtin (`File::*`,
// `Process::*`, `Env::*`), so this layer is pure API mapping with Go's
// names and value shapes. Import as `import std::os;`.

/// File metadata (Go's `os.FileInfo`, subset).
pub struct FileInfo {
    name: String,
    size: i64,
    is_dir: bool,
}

/// Returns the command-line arguments (Go's `os.Args`).
pub fn Args() -> Dynamic {
    return Process::args();
}

/// Returns the value of `key`, or "" when unset (Go's `os.Getenv`).
pub fn Getenv(key: String) -> String {
    let v = Env::get(key);
    if v == "NOT_FOUND" {
        return "";
    }
    return v;
}

/// Binds `key` to `value` (Go's `os.Setenv`).
pub fn Setenv(key: String, value: String) -> void {
    Env::set(key, value);
}

/// Reports whether `key` is set, plus its value (Go's `os.LookupEnv`).
pub fn LookupEnv(key: String) -> Dynamic {
    if Env::exists(key) {
        return [Getenv(key), true];
    }
    return ["", false];
}

/// Terminates the process with `code` (Go's `os.Exit`).
pub fn Exit(code: i64) -> void {
    Process::exit(code);
}

/// Returns the current working directory (Go's `os.Getwd`).
pub fn Getwd() -> String {
    return File::getwd();
}

/// Returns the OS temporary directory (Go's `os.TempDir`).
pub fn TempDir() -> String {
    return File::temp_dir();
}

/// Returns the current user's home directory (Go's `os.UserHomeDir`).
pub fn UserHomeDir() -> String {
    return File::home_dir();
}

/// Returns the machine hostname (Go's `os.Hostname`).
pub fn Hostname() -> String {
    return Process::hostname();
}

/// Reads the whole file as a string; "" when unreadable. Prefer
/// `ReadFileOk` when you need the error (Go's `os.ReadFile` returns
/// `(data, err)`; the ok-flag is the struct adaptation).
pub fn ReadFile(path: String) -> String {
    return File::read_string(path);
}

/// Reads the whole file plus an `ok` flag (Go's `(data, err)` pair).
pub fn ReadFileOk(path: String) -> Dynamic {
    if File::exists(path) {
        return [File::read_string(path), true];
    }
    return ["", false];
}

/// Writes `data` to `path`, reporting success (Go's `os.WriteFile`).
pub fn WriteFile(path: String, data: String) -> bool {
    return File::write_string(path, data);
}

/// Reports whether `path` exists.
pub fn Exists(path: String) -> bool {
    return File::exists(path);
}

/// Creates one directory level (Go's `os.Mkdir`).
pub fn Mkdir(path: String) -> bool {
    return File::mkdir(path);
}

/// Creates `path` and parents (Go's `os.MkdirAll`).
pub fn MkdirAll(path: String) -> bool {
    return File::mkdir_all(path);
}

/// Removes a file (Go's `os.Remove`).
pub fn Remove(path: String) -> bool {
    return File::remove(path);
}

/// Lists entry names of `dir` (Go's `os.ReadDir`, names-only subset).
pub fn ReadDir(dir: String) -> Dynamic {
    return File::read_dir(dir);
}

/// Stats `path` (Go's `os.Stat`, subset — missing files report size -1).
pub fn Stat(path: String) -> FileInfo {
    let base = path;
    let i = Text::last_index_of(path, "/");
    if i >= 0 {
        base = Text::slice(path, i + 1, path.len());
    }
    return FileInfo { name: base, size: File::size(path), is_dir: File::is_dir(path) };
}
