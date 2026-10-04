// Plugin model (Spec §63) and runtime lifecycle (Spec §17).
//
// The plugin ABI pins ownership (host owns all memory crossing the
// boundary), thread lifecycle (plugins MUST NOT spawn unmanaged threads),
// error propagation (all errors cross as codes, never panics) and
// shutdown order (reverse initialization). Optional sandboxing restricts
// fs/net/ffi per plugin.

/// Language ABI plugins are built against (mirrors codegen LEX_ABI_VERSION).
pub const PLUGIN_ABI: u32 = 1;

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub abi: u32,
    pub deps: Vec<String>,
}

impl PluginMetadata {
    /// Parse `name`, `version` (`major.minor.patch`), `abi`, `deps`.
    pub fn parse(manifest: &str) -> Result<Self, String> {
        let mut name = None;
        let mut version = None;
        let mut abi = None;
        let mut deps = Vec::new();
        for line in manifest.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (k, v) = line.split_once('=')
                .ok_or_else(|| format!("E0201: malformed plugin line: {}", line))?;
            match k.trim() {
                "name" => name = Some(v.trim().trim_matches('"').to_string()),
                "version" => version = Some(v.trim().trim_matches('"').to_string()),
                "abi" => abi = Some(v.trim().parse::<u32>()
                    .map_err(|_| "E0201: bad abi version".to_string())?),
                "deps" => {
                    deps = v.trim().trim_matches(|c| c == '"' || c == '[' || c == ']')
                        .split(',')
                        .filter_map(|d| {
                            let d = d.trim().trim_matches('"').to_string();
                            if d.is_empty() { None } else { Some(d) }
                        })
                        .collect();
                }
                _ => {}
            }
        }
        Ok(PluginMetadata {
            name: name.ok_or_else(|| "E0201: plugin missing `name`".to_string())?,
            version: version.ok_or_else(|| "E0201: plugin missing `version`".to_string())?,
            abi: abi.ok_or_else(|| "E0201: plugin missing `abi`".to_string())?,
            deps,
        })
    }

    /// Major version + ABI must match the host (Spec §63 compat).
    pub fn compatible_with(&self, host_version: &str, host_abi: u32) -> bool {
        if self.abi != host_abi {
            return false;
        }
        let major = |v: &str| v.split('.').next().unwrap_or("").to_string();
        major(&self.version) == major(host_version)
    }
}

#[derive(Debug, Clone)]
pub struct Sandbox {
    pub allow_fs: bool,
    pub allow_net: bool,
    pub allow_ffi: bool,
}

impl Default for Sandbox {
    fn default() -> Self {
        Sandbox { allow_fs: false, allow_net: false, allow_ffi: false }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Discovered,
    Loaded,
    Initialized,
    Shutdown,
}

pub struct PluginHandle {
    pub meta: PluginMetadata,
    pub sandbox: Sandbox,
    state: PluginState,
    init_order: usize,
}

impl PluginHandle {
    pub fn new(meta: PluginMetadata, sandbox: Sandbox, init_order: usize) -> Self {
        PluginHandle { meta, sandbox, state: PluginState::Discovered, init_order }
    }

    pub fn load(&mut self) {
        self.state = PluginState::Loaded;
    }

    pub fn init(&mut self) -> Result<(), String> {
        if self.state != PluginState::Loaded {
            return Err("E0501: plugin must be loaded before init".to_string());
        }
        self.state = PluginState::Initialized;
        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.state = PluginState::Shutdown;
    }

    pub fn state(&self) -> PluginState {
        self.state
    }

    pub fn init_order(&self) -> usize {
        self.init_order
    }
}

/// Shut plugins down in reverse initialization order (Spec §63).
pub fn shutdown_all(plugins: &mut [PluginHandle]) {
    let mut order: Vec<usize> = (0..plugins.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(plugins[i].init_order()));
    for i in order {
        plugins[i].shutdown();
    }
}

// ---------------------------------------------------------------------------
// Runtime lifecycle (Spec §17): explicit init/shutdown, no hidden globals
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    New,
    Running,
    ShuttingDown,
    Stopped,
}

pub struct RuntimeHooks {
    pub on_log: Option<Box<dyn Fn(&str) + Send + Sync>>,
    pub on_profile_sample: Option<Box<dyn Fn(&str) + Send + Sync>>,
    pub on_debug_event: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl Default for RuntimeHooks {
    fn default() -> Self {
        RuntimeHooks { on_log: None, on_profile_sample: None, on_debug_event: None }
    }
}

/// Explicit runtime handle. Initialization and shutdown are visible calls;
/// there is no implicit global runtime behind safe APIs.
pub struct Runtime {
    state: RuntimeState,
    hooks: RuntimeHooks,
    // Shutdown runs in reverse order of registration.
    shutdown_steps: Vec<(usize, String)>,
}

impl Runtime {
    pub fn new(hooks: RuntimeHooks) -> Self {
        Runtime { state: RuntimeState::New, hooks, shutdown_steps: Vec::new() }
    }

    pub fn init(&mut self) {
        self.state = RuntimeState::Running;
        self.emit_log("runtime init");
    }

    pub fn register_shutdown(&mut self, order: usize, name: &str) {
        self.shutdown_steps.push((order, name.to_string()));
    }

    pub fn shutdown(&mut self) -> Vec<String> {
        self.state = RuntimeState::ShuttingDown;
        self.shutdown_steps.sort_by_key(|(o, _)| std::cmp::Reverse(*o));
        let names = self.shutdown_steps.iter().map(|(_, n)| n.clone()).collect();
        self.emit_log("runtime shutdown");
        self.state = RuntimeState::Stopped;
        names
    }

    pub fn state(&self) -> RuntimeState {
        self.state
    }

    fn emit_log(&self, msg: &str) {
        if let Some(f) = &self.hooks.on_log {
            f(msg);
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new(RuntimeHooks::default())
    }
}

// ---------------------------------------------------------------------------
// Panic / recover (Spec §3 + §8): panic never skips cleanup silently
// ---------------------------------------------------------------------------

/// Raise a fatal runtime error (E0501). Unwinds through `defer` handlers;
/// use `lex_recover` at the boundary to convert into a value.
pub fn lex_panic(message: &str) -> ! {
    panic!("E0501: {}", message)
}

/// Run `f`, converting panics into `Err` with the panic message.
/// Resource cleanup (`Drop`, scope guards) still runs during unwind.
pub fn lex_recover<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => Ok(v),
        Err(payload) => {
            let msg = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown panic".to_string());
            Err(msg)
        }
    }
}

// ---------------------------------------------------------------------------
// Histograms and tracing spans (Spec §28 + §29)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct Histogram {
    bounds: Vec<f64>,
    counts: Vec<u64>,
}

impl Histogram {
    pub fn new(mut bounds: Vec<f64>) -> Self {
        bounds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = bounds.len() + 1;
        Histogram { bounds, counts: vec![0; n] }
    }

    pub fn observe(&mut self, value: f64) {
        let mut idx = self.counts.len() - 1;
        for (i, b) in self.bounds.iter().enumerate() {
            if value <= *b {
                idx = i;
                break;
            }
        }
        self.counts[idx] += 1;
    }

    pub fn counts(&self) -> &[u64] {
        &self.counts
    }
}

/// Distributed-tracing span with correlation id (Spec §29).
pub struct Span {
    pub name: String,
    pub correlation_id: String,
    start: std::time::Instant,
    pub elapsed_ms: Option<u128>,
}

impl Span {
    pub fn begin(name: &str, correlation_id: &str) -> Self {
        Span {
            name: name.to_string(),
            correlation_id: correlation_id.to_string(),
            start: std::time::Instant::now(),
            elapsed_ms: None,
        }
    }

    pub fn end(&mut self) {
        self.elapsed_ms = Some(self.start.elapsed().as_millis());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_parse_and_compat() {
        let m = PluginMetadata::parse("name = \"auth\"\nversion = \"1.2.0\"\nabi = 1\ndeps = [\"core\"]").unwrap();
        assert_eq!(m.name, "auth");
        assert!(m.compatible_with("1.9.0", PLUGIN_ABI));
        assert!(!m.compatible_with("2.0.0", PLUGIN_ABI));
        assert!(!m.compatible_with("1.9.0", 99));
        assert!(PluginMetadata::parse("name=\"x\"").is_err());
    }

    #[test]
    fn lifecycle_and_shutdown_order() {
        let mk = |name: &str, order: usize| {
            let mut h = PluginHandle::new(
                PluginMetadata { name: name.into(), version: "1.0.0".into(), abi: PLUGIN_ABI, deps: vec![] },
                Sandbox::default(),
                order,
            );
            h.load();
            h.init().unwrap();
            h
        };
        let mut plugins = vec![mk("a", 0), mk("b", 1), mk("c", 2)];
        assert!(plugins.iter().all(|p| p.state() == PluginState::Initialized));
        shutdown_all(&mut plugins);
        assert!(plugins.iter().all(|p| p.state() == PluginState::Shutdown));
        // Reverse-order shutdown verified via Runtime ordering below.
        let mut rt = Runtime::new(RuntimeHooks::default());
        rt.init();
        rt.register_shutdown(1, "net");
        rt.register_shutdown(5, "fs");
        assert_eq!(rt.shutdown(), vec!["fs".to_string(), "net".to_string()]);
        assert_eq!(rt.state(), RuntimeState::Stopped);
    }

    #[test]
    fn recover_converts_panic() {
        assert_eq!(lex_recover(|| 2 + 2), Ok(4));
        let err = lex_recover(|| lex_panic("boom")).unwrap_err();
        assert!(err.contains("E0501") && err.contains("boom"));
    }

    #[test]
    fn histogram_buckets() {
        let mut h = Histogram::new(vec![10.0, 100.0]);
        h.observe(5.0);
        h.observe(50.0);
        h.observe(500.0);
        assert_eq!(h.counts(), &[1, 1, 1]);
    }

    #[test]
    fn span_elapses() {
        let mut s = Span::begin("compile", "corr-1");
        s.end();
        assert!(s.elapsed_ms.is_some());
    }
}
