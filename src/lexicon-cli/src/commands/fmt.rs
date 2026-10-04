use anyhow::Result;

/// Legacy stub entry — preserved for compatibility.
pub fn fmt_command() -> Result<()> {
    println!("Formatting code...");
    Ok(())
}

/// Deterministic formatter re-export (Spec §24).
/// Block-AST reindentation lives in `compiler::format_source`; this
/// wrapper keeps `commands::fmt` in sync with `lex fmt`.
pub fn format_source_stable(source: &str) -> String {
    crate::compiler::format_source(source)
}
