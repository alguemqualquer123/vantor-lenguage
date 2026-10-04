use anyhow::Result;

/// Legacy stub entry — preserved for compatibility.
pub fn build_command() -> Result<()> {
    println!("Building project...");
    Ok(())
}

/// Reproducible-build helpers (Spec §43 partial).
/// The real SBOM/fingerprint writers live in `crate::compiler` so the
/// CLI has a single source of truth; these re-exports keep the
/// `commands::build` surface usable by scripts and tests.

/// Canonical SBOM output path (next to `build/fingerprint.txt`).
pub fn sbom_path() -> &'static str {
    "build/sbom.json"
}

/// Canonical fingerprint path (written by `compiler::build_with_target`).
pub fn fingerprint_path() -> &'static str {
    "build/fingerprint.txt"
}

/// Write the deterministic SBOM stub for `target`/`profile`.
pub fn write_sbom_stub(target: &str, profile: &str) -> Result<()> {
    crate::compiler::generate_sbom(target, profile)
}
