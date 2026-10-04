use wasm_bindgen::prelude::*;

// Crate version surfaced to both native hosts and JS guests.
pub const LEX_WASM_API_VERSION: &str = env!("CARGO_PKG_VERSION");

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// ---------------------------------------------------------------------------
// Part X / §42 + §61: WASM export helpers.
//
// These helpers describe what a compiled Lex module exposes to a WASM
// host (export names, MVP memory budget, API version) without requiring
// a WASM runtime: playground tooling, game engines embedding scripts,
// and the AOT path plan through these shapes on the native host.
// Real `.wasm` emission stays in codegen; `lexicon-wasm` owns the
// guest/host boundary naming.
// ---------------------------------------------------------------------------

/// Lex toolchain API version visible to WASM guests.
#[wasm_bindgen]
pub fn lex_version() -> String {
    LEX_WASM_API_VERSION.to_string()
}

/// `true` when running as a `wasm32` guest.
pub fn is_wasm_target() -> bool {
    cfg!(target_arch = "wasm32")
}

/// WASM MVP triple served by the backend.
pub fn wasm_triple() -> &'static str {
    "wasm32-unknown-unknown"
}

/// Stable export name for `module::name` (matches the utils-level
/// `wasm_export_name` so host and guest agree byte-for-byte).
pub fn export_name(module: &str, name: &str) -> String {
    let clean = |s: &str| {
        s.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>()
    };
    format!("{}_{}", clean(module), clean(name))
}

/// `true` when `byte_len` fits the MVP memory budget (`pages` × 64 KiB).
pub fn fits_memory(byte_len: usize, pages: u32) -> bool {
    byte_len as u64 <= pages as u64 * 65536
}

/// Machine-readable export descriptor for hosts/registries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmExport {
    pub module: String,
    pub name: String,
    pub kind: String,
}

impl WasmExport {
    pub fn new(module: impl Into<String>, name: impl Into<String>, kind: impl Into<String>) -> Self {
        WasmExport { module: module.into(), name: name.into(), kind: kind.into() }
    }

    /// Guest-visible symbol for this export.
    pub fn symbol(&self) -> String {
        export_name(&self.module, &self.name)
    }

    /// Minimal JSON rendering (no extra deps; hosts parse it directly).
    pub fn to_json(&self) -> String {
        let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "{{\"module\":\"{}\",\"name\":\"{}\",\"kind\":\"{}\",\"symbol\":\"{}\"}}",
            esc(&self.module),
            esc(&self.name),
            esc(&self.kind),
            esc(&self.symbol()),
        )
    }
}

/// JSON array of export descriptors (host/registry ingestion).
pub fn describe_exports(exports: &[WasmExport]) -> String {
    let parts: Vec<String> = exports.iter().map(|e| e.to_json()).collect();
    format!("[{}]", parts.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_naming_matches_host_convention() {
        assert_eq!(export_name("my mod", "run!"), "my_mod_run_");
        let e = WasmExport::new("app", "main", "func");
        assert_eq!(e.symbol(), "app_main");
        assert!(e.to_json().contains("\"symbol\":\"app_main\""));
        assert_eq!(
            describe_exports(&[e]),
            "[{\"module\":\"app\",\"name\":\"main\",\"kind\":\"func\",\"symbol\":\"app_main\"}]"
        );
    }

    #[test]
    fn memory_budget_and_version() {
        assert!(fits_memory(1024, 256));
        assert!(!fits_memory(usize::MAX, 1));
        assert_eq!(wasm_triple(), "wasm32-unknown-unknown");
        assert!(!lex_version().is_empty());
        let _ = is_wasm_target();
    }
}
