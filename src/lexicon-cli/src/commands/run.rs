use anyhow::Result;

/// Legacy stub entry — preserved for compatibility.
pub fn run_command(args: Vec<String>) -> Result<()> {
    println!("Running project with args: {:?}", args);
    Ok(())
}

/// CI-mode probe for `commands::run` callers (Spec §23).
/// True when `--ci` was passed through or `CI=true` is in the env.
pub fn is_ci(ci_flag: bool) -> bool {
    crate::compiler::is_ci_mode(ci_flag)
}

/// Non-interactive runner used by CI pipelines.
/// Delegates to `compiler::run_with_ci` so behaviour matches `lex run`.
pub fn run_command_with_ci(file: Option<String>, args: Vec<String>, ci: bool) -> Result<()> {
    crate::compiler::run_with_ci(file, args, ci)
}
