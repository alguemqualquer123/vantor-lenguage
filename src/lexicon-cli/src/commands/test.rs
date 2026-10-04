use anyhow::Result;

/// Legacy stub entry — preserved for compatibility.
pub fn test_command() -> Result<()> {
    println!("Running tests...");
    Ok(())
}

/// Fuzzing hook re-export (Spec §43 partial).
/// Feeds arbitrary bytes into the lex/parse pipeline; `true` means the
/// pipeline survived without panicking. Malformed input must never crash.
pub fn fuzz_target(data: &[u8]) -> bool {
    crate::compiler::fuzz_target(data)
}

/// Benchmark-suite header helper (Spec §45 partial + §71 partial).
/// Formats the flags/target/hw-sw line recorded by `compiler::bench_*`
/// so external runners can reuse the exact repeatability metadata.
pub fn bench_header(target: &str, profile: &str, flags: &str) -> String {
    format!(
        "target={} profile={} flags={} hw={} {} sw=lexc=0.2.0",
        target,
        profile,
        flags,
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}

/// Security-scan entry for test pipelines (Spec §43 partial).
/// Scans `file`/`source` and returns the number of findings.
pub fn security_scan(file: &str, source: &str) -> usize {
    crate::compiler::analyze_security_source(file, source).len()
}
