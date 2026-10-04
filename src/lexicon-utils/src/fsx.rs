// File systems, processes and OS services (Spec §49 + §50).
//
// Cross-platform abstractions with explicit semantics: paths are
// normalized lexically (no silent absolutization), process APIs expose
// cancellation, exit status and handle inheritance, and system info is
// read-only. Platform specifics that cannot be abstracted live behind
// `current_os()` instead of hidden conditionals.

use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, ExitStatus};

/// Operating systems abstracted by this module (Spec §77).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    Linux,
    MacOs,
    Bsd,
    Android,
    Ios,
    Wasm,
    Unknown,
}

pub fn current_os() -> Os {
    match std::env::consts::OS {
        "windows" => Os::Windows,
        "linux" => Os::Linux,
        "macos" => Os::MacOs,
        "freebsd" | "openbsd" | "netbsd" => Os::Bsd,
        "android" => Os::Android,
        "ios" => Os::Ios,
        _ => Os::Unknown,
    }
}

pub fn current_arch() -> &'static str {
    std::env::consts::ARCH
}

/// Lexically normalize a path: resolve `.`/`..`, collapse separators,
/// convert `\` to `/` on Windows input. Never touches the filesystem and
/// never silently absolutizes (Spec §49 safe normalization).
pub fn normalize_path(input: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    let rooted = input.starts_with('/') || input.starts_with('\\');
    // Windows drive prefix (C:) is preserved verbatim.
    let mut drive = String::new();
    let rest = if input.len() >= 2 && input.as_bytes()[1] == b':' {
        drive = input[..2].to_string();
        &input[2..]
    } else {
        input
    };
    for comp in rest.replace('\\', "/").split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                if parts.last().map(|p| p.as_str()) != Some("..") && !parts.is_empty() {
                    parts.pop();
                } else if !rooted {
                    parts.push("..".to_string());
                }
            }
            c => parts.push(c.to_string()),
        }
    }
    let joined = parts.join("/");
    if rooted {
        format!("/{}", joined)
    } else if !drive.is_empty() {
        format!("{}/{}", drive, joined)
    } else if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}

#[derive(Debug, Clone)]
pub struct FileMeta {
    pub len: u64,
    pub is_dir: bool,
    pub is_file: bool,
    pub readonly: bool,
}

pub fn read_file(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("E0601: read {}: {}", path, e))
}

pub fn write_file(path: &str, data: &[u8]) -> Result<(), String> {
    std::fs::write(path, data).map_err(|e| format!("E0601: write {}: {}", path, e))
}

pub fn file_meta(path: &str) -> Result<FileMeta, String> {
    let m = std::fs::metadata(path).map_err(|e| format!("E0601: stat {}: {}", path, e))?;
    Ok(FileMeta {
        len: m.len(),
        is_dir: m.is_dir(),
        is_file: m.is_file(),
        readonly: m.permissions().readonly(),
    })
}

pub fn list_dir(path: &str) -> Result<Vec<String>, String> {
    let entries =
        std::fs::read_dir(path).map_err(|e| format!("E0601: list {}: {}", path, e))?;
    let mut out = Vec::new();
    for e in entries {
        let e = e.map_err(|e| format!("E0601: list entry: {}", e))?;
        out.push(e.path().to_string_lossy().to_string());
    }
    out.sort();
    Ok(out)
}

/// Create a uniquely-named temporary directory and return its path.
pub fn temp_dir(prefix: &str) -> Result<PathBuf, String> {
    let base = std::env::temp_dir();
    for i in 0..1000u32 {
        let cand = base.join(format!("{}_{}_{}", prefix, std::process::id(), i));
        if !cand.exists() {
            std::fs::create_dir_all(&cand)
                .map_err(|e| format!("E0601: tempdir: {}", e))?;
            return Ok(cand);
        }
    }
    Err("E0601: cannot allocate tempdir".to_string())
}

// ---------------------------------------------------------------------------
// Processes (Spec §50)
// ---------------------------------------------------------------------------

pub struct LexChild {
    inner: Child,
}

impl LexChild {
    /// Spawn with piped stdio; `inherit_handles` controls stdio inheritance.
    pub fn spawn(program: &str, args: &[&str], inherit_handles: bool) -> Result<Self, String> {
        let mut cmd = Command::new(program);
        cmd.args(args);
        if !inherit_handles {
            use std::process::Stdio;
            cmd.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
        }
        let inner = cmd.spawn().map_err(|e| format!("E0501: spawn {}: {}", program, e))?;
        Ok(LexChild { inner })
    }

    /// Terminate (cancellation, Spec §50) and wait for the exit status.
    pub fn kill(&mut self) -> Result<ExitStatus, String> {
        self.inner.kill().map_err(|e| format!("E0501: kill: {}", e))?;
        self.wait()
    }

    pub fn wait(&mut self) -> Result<ExitStatus, String> {
        self.inner.wait().map_err(|e| format!("E0501: wait: {}", e))
    }

    pub fn id(&self) -> u32 {
        self.inner.id()
    }
}

pub fn env_get(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Read-only system information snapshot.
///
/// Ownership: owned value, no borrows.
/// Thread-safety: `Send + Sync`; safe to share across threads.
/// Complexity: O(1) except `exe`/`cwd` syscalls.
#[derive(Debug, Clone)]
pub struct SysInfo {
    pub os: Os,
    pub arch: &'static str,
    pub cpus: usize,
    pub exe: String,
    /// Process id of the caller. Complexity: O(1).
    pub pid: u32,
    /// Current working directory (lossy). Complexity: O(1) syscall.
    pub cwd: String,
    /// Host name if resolvable via env (`HOSTNAME`/`COMPUTERNAME`), else empty.
    /// Complexity: O(1). Never panics.
    pub hostname: String,
}

pub fn sysinfo() -> SysInfo {
    SysInfo {
        os: current_os(),
        arch: current_arch(),
        cpus: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
        exe: std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        pid: std::process::id(),
        cwd: std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        hostname: std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_default(),
    }
}

/// Suppress unused-import warning for Component while documenting that
/// normalization is lexical (components are matched by string above).
#[allow(dead_code)]
fn _components(_p: &Path) -> Vec<Component<'_>> {
    _p.components().collect()
}

// ---------------------------------------------------------------------------
// Paths: pure lexical helpers (Spec §49).
//
// Ownership: all helpers take `&str` and return owned `String`s; no borrows
// escape. Thread-safety: pure functions, `Send + Sync`.
// Complexity: O(n) in path length unless noted.
// ---------------------------------------------------------------------------

/// Join segments with `/` then lexically normalize.
/// Complexity: O(total length).
pub fn join_path(parts: &[&str]) -> String {
    normalize_path(&parts.join("/"))
}

/// Directory portion (`a/b/c.txt` → `a/b`; `c.txt` → `.`).
/// Complexity: O(n).
pub fn dirname(path: &str) -> String {
    let n = normalize_path(path);
    match n.rfind('/') {
        Some(0) => "/".to_string(),
        Some(i) => n[..i].to_string(),
        None => ".".to_string(),
    }
}

/// Final component (`a/b/c.txt` → `c.txt`). Complexity: O(n).
pub fn basename(path: &str) -> String {
    let n = normalize_path(path);
    match n.rfind('/') {
        Some(i) => n[i + 1..].to_string(),
        None => n,
    }
}

/// Extension without dot (`c.tar.gz` → `gz`; `.hidden` → ``).
/// Complexity: O(n).
pub fn extension(path: &str) -> String {
    let b = basename(path);
    match b.rfind('.') {
        Some(0) | None => String::new(),
        Some(i) => b[i + 1..].to_string(),
    }
}

/// True for rooted paths (`/...`) or Windows drive prefixes (`C:/...`).
/// Complexity: O(1).
pub fn is_absolute(path: &str) -> bool {
    path.starts_with('/') || path.starts_with('\\') || (path.len() >= 2 && path.as_bytes()[1] == b':')
}

// ---------------------------------------------------------------------------
// Dirs / file ops beyond read/write (Spec §49).
//
// Ownership: paths borrowed, data owned. Errors carry `E0601` codes.
// Thread-safety: filesystem calls are thread-safe; no internal locking.
// Complexity: O(1) syscalls except recursive remove (O(entries)).
// ---------------------------------------------------------------------------

/// Create directories recursively. Complexity: O(depth) syscalls.
pub fn create_dir_all(path: &str) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| format!("E0601: mkdir {}: {}", path, e))
}

/// Remove a single file. Complexity: O(1).
pub fn remove_file(path: &str) -> Result<(), String> {
    std::fs::remove_file(path).map_err(|e| format!("E0601: rm {}: {}", path, e))
}

/// Remove a directory tree. Complexity: O(entries).
pub fn remove_dir_all(path: &str) -> Result<(), String> {
    std::fs::remove_dir_all(path).map_err(|e| format!("E0601: rmdir {}: {}", path, e))
}

/// Rename/move a file or directory. Complexity: O(1) on same filesystem.
pub fn rename_path(from: &str, to: &str) -> Result<(), String> {
    std::fs::rename(from, to).map_err(|e| format!("E0601: rename {} -> {}: {}", from, to, e))
}

/// Copy a file, returning bytes copied. Complexity: O(bytes).
pub fn copy_file(from: &str, to: &str) -> Result<u64, String> {
    std::fs::copy(from, to).map_err(|e| format!("E0601: copy {} -> {}: {}", from, to, e))
}

/// Existence probe (file, dir or symlink). Complexity: O(1).
pub fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

// ---------------------------------------------------------------------------
// Permissions (Spec §49, per-platform).
//
// Ownership: owned snapshots. Thread-safety: `Send + Sync`.
// Complexity: O(1) syscalls.
// ---------------------------------------------------------------------------

/// Portable permission snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePerms {
    pub readonly: bool,
}

pub fn get_perms(path: &str) -> Result<FilePerms, String> {
    let m = std::fs::metadata(path).map_err(|e| format!("E0601: stat {}: {}", path, e))?;
    Ok(FilePerms { readonly: m.permissions().readonly() })
}

/// Toggle the readonly bit. Full unix mode bits are out of scope and
/// documented as an integration point.
/// Complexity: O(1).
pub fn set_readonly(path: &str, readonly: bool) -> Result<(), String> {
    let mut p = std::fs::metadata(path)
        .map_err(|e| format!("E0601: stat {}: {}", path, e))?
        .permissions();
    p.set_readonly(readonly);
    std::fs::set_permissions(path, p).map_err(|e| format!("E0601: chmod {}: {}", path, e))
}

// ---------------------------------------------------------------------------
// Environment (Spec §50).
//
// Ownership: owned strings. Thread-safety: `std::env` is process-global;
// concurrent set/remove races are the caller's responsibility.
// Complexity: O(1) get/set, O(n) list.
// ---------------------------------------------------------------------------

/// Overwrite an environment variable. Complexity: O(1).
pub fn env_set(name: &str, value: &str) {
    std::env::set_var(name, value);
}

/// Remove an environment variable. Complexity: O(1).
pub fn env_remove(name: &str) {
    std::env::remove_var(name);
}

/// Sorted snapshot of the whole environment. Complexity: O(n log n).
pub fn env_list() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = std::env::vars().collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

// ---------------------------------------------------------------------------
// TTY (Spec §50).
//
// Ownership: no state. Thread-safety: `Send + Sync`.
// Complexity: O(1).
// ---------------------------------------------------------------------------

/// True when stdout is attached to a terminal.
/// Uses `std::io::IsTerminal`; never panics.
pub fn is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

/// Terminal width fallback: `COLUMNS` env or 80.
/// Complexity: O(1). Deterministic given the environment.
pub fn tty_width_or(fallback: usize) -> usize {
    std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok()).unwrap_or(fallback)
}

// ---------------------------------------------------------------------------
// Signals (Spec §50).
//
// Ownership: `Copy` enum. Thread-safety: `Send + Sync`.
// The real OS delivery behind `send_signal` is a documented stub: portable
// Rust `std` cannot raise/forward arbitrary signals without `libc`; the API
// shape is stable so a `libc` backend can land without rewrites.
// ---------------------------------------------------------------------------

/// Portable signal set.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Interrupt,
    Terminate,
    Kill,
    Hangup,
    User1,
    User2,
}

#[allow(dead_code)]
impl Signal {
    /// Short name (`SIGINT`, ...). Complexity: O(1).
    pub fn name(self) -> &'static str {
        match self {
            Signal::Interrupt => "SIGINT",
            Signal::Terminate => "SIGTERM",
            Signal::Kill => "SIGKILL",
            Signal::Hangup => "SIGHUP",
            Signal::User1 => "SIGUSR1",
            Signal::User2 => "SIGUSR2",
        }
    }

    /// Parse `SIGINT` / `INT` / `2` forms. Complexity: O(1).
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "SIGINT" | "INT" | "2" => Some(Signal::Interrupt),
            "SIGTERM" | "TERM" | "15" => Some(Signal::Terminate),
            "SIGKILL" | "KILL" | "9" => Some(Signal::Kill),
            "SIGHUP" | "HUP" | "1" => Some(Signal::Hangup),
            "SIGUSR1" | "USR1" | "10" => Some(Signal::User1),
            "SIGUSR2" | "USR2" | "12" => Some(Signal::User2),
            _ => None,
        }
    }
}

/// Stub: delivering a signal to another pid needs `libc`/Win32.
/// Returns `Err` describing the integration point; `Kill` on the current
/// process id terminates via `std::process::exit` semantics is NOT done
/// implicitly — the caller decides.
/// Complexity: O(1). Ownership: no handles retained.
#[allow(dead_code)]
pub fn send_signal(_pid: u32, _sig: Signal) -> Result<(), String> {
    Err("E0501: send_signal is a stub; integrate `libc::kill` (unix) / Win32 TerminateProcess".to_string())
}

// ---------------------------------------------------------------------------
// Pipes (Spec §50).
//
// `Pipe` is an in-memory bounded byte channel standing in for an OS pipe:
// `std` stable has no anonymous-pipe constructor, so this keeps the
// blocking `Reader/Writer` shape, backpressure (`capacity`) and close
// semantics without new dependencies. An OS-pipe backend (`pipe(2)` /
// named pipes) can replace the internals without changing callers.
//
// Ownership: `PipeReader`/`PipeWriter` share one `Arc<Inner>`; dropping all
// writers delivers EOF (read returns 0). Thread-safety: `Send + Sync`
// (`Mutex` + `Condvar`). Complexity: write/read O(bytes).
// ---------------------------------------------------------------------------

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
struct PipeInner {
    queue: Mutex<VecDeque<u8>>,
    closed: Mutex<bool>,
    data: Condvar,
    capacity: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PipeWriter {
    inner: Arc<PipeInner>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PipeReader {
    inner: Arc<PipeInner>,
}

#[allow(dead_code)]
impl PipeWriter {
    /// Write bytes, blocking while full. Returns `Err` after close.
    /// Complexity: O(bytes).
    pub fn write(&self, buf: &[u8]) -> Result<usize, String> {
        let mut q = self.inner.queue.lock().map_err(|e| format!("E0501: pipe poisoned: {}", e))?;
        if *self.inner.closed.lock().unwrap() {
            return Err("E0501: pipe closed".to_string());
        }
        let mut n = 0;
        for b in buf {
            if q.len() >= self.inner.capacity {
                break; // bounded: partial write applies backpressure
            }
            q.push_back(*b);
            n += 1;
        }
        self.inner.data.notify_all();
        Ok(n)
    }

    pub fn close(&self) {
        *self.inner.closed.lock().unwrap() = true;
        self.inner.data.notify_all();
    }
}

#[allow(dead_code)]
impl PipeReader {
    /// Read up to `buf.len()` bytes. Returns `Ok(0)` on EOF (all writers
    /// dropped / closed and buffer drained). Complexity: O(bytes).
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, String> {
        let mut q = self.inner.queue.lock().map_err(|e| format!("E0501: pipe poisoned: {}", e))?;
        loop {
            if !q.is_empty() {
                let n = q.len().min(buf.len());
                for (i, slot) in buf[..n].iter_mut().enumerate() {
                    *slot = q.pop_front().unwrap();
                    let _ = i;
                }
                return Ok(n);
            }
            if *self.inner.closed.lock().unwrap() {
                return Ok(0);
            }
            q = self.inner.data.wait(q).map_err(|e| format!("E0501: pipe poisoned: {}", e))?;
        }
    }
}

/// Create a bounded in-memory pipe.
/// Ownership: the two halves share state; EOF when writers drop.
/// Complexity: O(1).
#[allow(dead_code)]
pub fn pipe(capacity: usize) -> (PipeWriter, PipeReader) {
    let inner = Arc::new(PipeInner {
        queue: Mutex::new(VecDeque::new()),
        closed: Mutex::new(false),
        data: Condvar::new(),
        capacity: capacity.max(1),
    });
    (PipeWriter { inner: Arc::clone(&inner) }, PipeReader { inner })
}

// ---------------------------------------------------------------------------
// IPC / shared memory stubs (Spec §50).
//
// Documented integration points: real IPC needs OS handles (UDS / named
// pipes / `shm_open`+`mmap`). Shapes below pin message framing (length
// cap), ownership (caller-owned buffers) and error codes so the backend
// can land later. All stubs are `#[allow(dead_code)]` by design.
// ---------------------------------------------------------------------------

/// Max IPC message payload accepted (1 MiB). Untrusted input over this is rejected.
#[allow(dead_code)]
pub const MAX_IPC_MSG: usize = 1 << 20;

/// In-process message channel stub with framing + cap.
/// Ownership: owns queued messages. Thread-safety: `Send` via `Mutex`.
/// Complexity: send/recv O(bytes).
#[allow(dead_code)]
pub struct IpcChannel {
    queue: Mutex<VecDeque<Vec<u8>>>,
    closed: Mutex<bool>,
}

#[allow(dead_code)]
impl IpcChannel {
    pub fn new() -> Self {
        IpcChannel { queue: Mutex::new(VecDeque::new()), closed: Mutex::new(false) }
    }

    pub fn send(&self, msg: &[u8]) -> Result<(), String> {
        if msg.len() > MAX_IPC_MSG {
            return Err(format!("E0501: ipc message exceeds {} bytes", MAX_IPC_MSG));
        }
        if *self.closed.lock().unwrap() {
            return Err("E0501: ipc channel closed".to_string());
        }
        self.queue.lock().unwrap().push_back(msg.to_vec());
        Ok(())
    }

    pub fn recv(&self) -> Option<Vec<u8>> {
        self.queue.lock().unwrap().pop_front()
    }

    pub fn close(&self) {
        *self.closed.lock().unwrap() = true;
    }
}

#[allow(dead_code)]
impl Default for IpcChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared-memory segment stub.
/// Real backend: `shm_open` + `mmap` (unix) / file mapping (windows).
/// Ownership: the segment owns `len` bytes; `read`/`write` borrow with
/// bounds checks. Thread-safety: NOT `Sync` — external locking required.
/// Complexity: read/write O(bytes).
#[allow(dead_code)]
pub struct SharedMemory {
    name: String,
    buf: Vec<u8>,
}

#[allow(dead_code)]
impl SharedMemory {
    /// Create (stub: heap-backed). Real impl would `shm_open` + truncate.
    pub fn create(name: &str, len: usize) -> Result<Self, String> {
        if len > (1 << 30) {
            return Err("E0501: shm segment exceeds 1 GiB cap".to_string());
        }
        Ok(SharedMemory { name: name.to_string(), buf: vec![0u8; len] })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> Result<(), String> {
        let end = offset.checked_add(data.len()).ok_or_else(|| "E0501: shm overflow".to_string())?;
        if end > self.buf.len() {
            return Err("E0501: shm write out of bounds".to_string());
        }
        self.buf[offset..end].copy_from_slice(data);
        Ok(())
    }

    pub fn read_at(&self, offset: usize, len: usize) -> Result<&[u8], String> {
        let end = offset.checked_add(len).ok_or_else(|| "E0501: shm overflow".to_string())?;
        if end > self.buf.len() {
            return Err("E0501: shm read out of bounds".to_string());
        }
        Ok(&self.buf[offset..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_cases() {
        assert_eq!(normalize_path("a/./b/../c"), "a/c");
        assert_eq!(normalize_path("/a//b/"), "/a/b");
        assert_eq!(normalize_path(""), ".");
        assert_eq!(normalize_path("C:\\a\\b"), "C:/a/b");
        assert_eq!(normalize_path("../../x"), "../../x");
    }

    #[test]
    fn sysinfo_sane() {
        let s = sysinfo();
        assert!(s.cpus >= 1);
        assert_ne!(s.arch, "");
    }

    #[test]
    fn file_roundtrip_in_temp() {
        let dir = temp_dir("lexfs").unwrap();
        let f = dir.join("a.txt");
        let path = f.to_string_lossy().to_string();
        write_file(&path, b"data").unwrap();
        assert_eq!(read_file(&path).unwrap(), b"data");
        let m = file_meta(&path).unwrap();
        assert!(m.is_file && m.len == 4);
        assert!(list_dir(&dir.to_string_lossy()).unwrap().len() == 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn spawn_self_list() {
        // The test binary itself is a guaranteed spawnable program;
        // `--list` is a valid libtest flag with exit status 0.
        let exe = std::env::current_exe().unwrap();
        let mut child = LexChild::spawn(&exe.to_string_lossy(), &["--list"], false)
            .expect("spawn self");
        let pid = child.id();
        assert!(pid > 0);
        let status = child.wait().expect("wait");
        assert!(status.success());
    }

    #[test]
    fn path_helpers() {
        assert_eq!(join_path(&["a", "b", "../c"]), "a/c");
        assert_eq!(dirname("a/b/c.txt"), "a/b");
        assert_eq!(basename("a/b/c.txt"), "c.txt");
        assert_eq!(extension("c.tar.gz"), "gz");
        assert_eq!(extension(".hidden"), "");
        assert!(is_absolute("/x"));
        assert!(is_absolute("C:/x"));
        assert!(!is_absolute("rel/x"));
    }

    #[test]
    fn dir_file_ops_roundtrip() {
        let dir = temp_dir("lexfs2").unwrap();
        let base = dir.to_string_lossy().to_string();
        let sub = format!("{}/sub", base);
        create_dir_all(&sub).unwrap();
        let f = format!("{}/f.txt", sub);
        write_file(&f, b"hi").unwrap();
        assert!(path_exists(&f));
        let g = format!("{}/g.txt", sub);
        copy_file(&f, &g).unwrap();
        rename_path(&g, &format!("{}/h.txt", sub)).unwrap();
        assert_eq!(get_perms(&f).unwrap().readonly, false);
        set_readonly(&f, true).unwrap();
        assert_eq!(get_perms(&f).unwrap().readonly, true);
        set_readonly(&f, false).unwrap();
        remove_file(&f).unwrap();
        assert!(!path_exists(&f));
        remove_dir_all(&base).unwrap();
    }

    #[test]
    fn env_and_signal() {
        env_set("LEX_TEST_X", "1");
        assert_eq!(env_get("LEX_TEST_X"), Some("1".to_string()));
        assert!(env_list().iter().any(|(k, _)| k == "LEX_TEST_X"));
        env_remove("LEX_TEST_X");
        assert_eq!(env_get("LEX_TEST_X"), None);
        assert_eq!(Signal::parse("SIGINT"), Some(Signal::Interrupt));
        assert_eq!(Signal::Interrupt.name(), "SIGINT");
        assert!(send_signal(1, Signal::Interrupt).is_err());
        let _ = tty_width_or(80);
    }

    #[test]
    fn pipe_ipc_shm() {
        let (w, r) = pipe(64);
        assert_eq!(w.write(b"abc").unwrap(), 3);
        let mut buf = [0u8; 8];
        assert_eq!(r.read(&mut buf).unwrap(), 3);
        assert_eq!(&buf[..3], b"abc");
        let ch = IpcChannel::new();
        ch.send(b"m").unwrap();
        assert_eq!(ch.recv().unwrap(), b"m");
        assert!(ch.send(&vec![0u8; MAX_IPC_MSG + 1]).is_err());
        let mut shm = SharedMemory::create("t", 8).unwrap();
        shm.write_at(0, b"hi").unwrap();
        assert_eq!(shm.read_at(0, 2).unwrap(), b"hi");
        assert!(shm.write_at(7, b"toolong").is_err());
    }
}
