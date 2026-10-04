use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, anyhow, Context};

/// Auto-install run at every `lex` startup. Deliberately SILENT: it must
/// never print and never duplicate PATH entries when everything is
/// already in place. Verbose output belongs to `manual_install` only.
///
/// Beyond the binary, it owns the SDK: `~/.lexicon/sdk` is installed on
/// first run and re-exported whenever its `VERSION` differs from the
/// running toolchain (silent update — one small file read when current).
pub fn auto_install() -> Result<()> {
    let current_exe = env::current_exe()?;
    let exe_name = "lex";

    // 1. Get user's home and target bin directory
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| anyhow!("Could not find home directory"))?;

    let lexicon_home = PathBuf::from(home).join(".lexicon");
    let bin_dir = lexicon_home.join("bin");

    // Create directories if they don't exist
    fs::create_dir_all(&bin_dir)?;

    // 2. Copy current executable to bin directory (self-update, silent)
    let mut target_exe = bin_dir.join(exe_name);
    if cfg!(windows) {
        target_exe.set_extension("exe");
    }

    // Only copy if it's different or doesn't exist
    if !target_exe.exists() || fs::metadata(&current_exe)?.len() != fs::metadata(&target_exe).map(|m| m.len()).unwrap_or(0) {
        copy_exe_overwrite(&current_exe, &target_exe)?;
    }

    // 3. Add to PATH only if truly absent (checks the persistent store,
    // not just the possibly-stale process environment).
    if !is_in_path(&bin_dir) {
        add_to_path(&bin_dir)?;
    }

    // 4. SDK: install on first run, update on version drift (silent).
    ensure_sdk_quiet(&lexicon_home)?;

    Ok(())
}

/// Canonical per-user home (`~/.lexicon`).
pub fn lexicon_home() -> Result<PathBuf> {
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| anyhow!("Could not find home directory"))?;
    Ok(PathBuf::from(home).join(".lexicon"))
}

/// Canonical installed-SDK root (`~/.lexicon/sdk`).
pub fn installed_sdk_root() -> Result<PathBuf> {
    Ok(lexicon_home()?.join("sdk"))
}

/// Silent first-run SDK reconciler: exports when `VERSION` is missing or
/// stale. One file read when current — negligible per-run cost.
pub fn ensure_sdk_quiet(home: &Path) -> Result<()> {
    let sdk_root = home.join("sdk");
    let current = crate::sdk::installed_version(&sdk_root);
    if current.as_deref() == Some(crate::sdk::SDK_VERSION) {
        return Ok(());
    }
    crate::sdk::export_to(&sdk_root, false)?;
    Ok(())
}

pub fn manual_install() -> Result<()> {
    // 1. Try to find the binary in common build locations
    let current_dir = env::current_dir()?;
    let exe_name = if cfg!(windows) { "lex.exe" } else { "lex" };
    
    let possible_paths = vec![
        env::current_exe()?,
        current_dir.join("target").join("release").join(exe_name),
        current_dir.join("target").join("debug").join(exe_name),
        current_dir.join(exe_name),
    ];

    let mut source_exe = None;
    for path in possible_paths {
        if path.exists() && path.is_file() {
            source_exe = Some(path);
            break;
        }
    }

    let source_exe = source_exe.with_context(|| "Could not find 'lex' binary to install. Please build the project first with 'cargo build --release'")?;

    // 2. Setup destination
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| anyhow!("Could not find home directory"))?;
    
    let lexicon_home = PathBuf::from(home).join(".lexicon");
    let bin_dir = lexicon_home.join("bin");
    fs::create_dir_all(&bin_dir)?;

    let target_exe = bin_dir.join(exe_name);

    // 3. Copy binary
    println!("📦 Copying binary from {:?} to {:?}", source_exe, target_exe);
    copy_exe_overwrite(&source_exe, &target_exe)?;

    // 4. Update PATH
    if !is_in_path(&bin_dir) {
        add_to_path(&bin_dir)?;
        println!("✅ Lexicon added to system PATH successfully!");
    } else {
        println!("✨ Lexicon is already in your system PATH.");
    }

    // 5. SDK (verbose here — `lex install` is the explicit moment).
    let sdk_root = lexicon_home.join("sdk");
    match crate::sdk::installed_version(&sdk_root) {
        Some(v) if v == crate::sdk::SDK_VERSION => {
            println!("✨ SDK {} already installed at {}.", v, sdk_root.display());
        }
        other => {
            if let Some(v) = other {
                println!("📦 Updating SDK {} → {} at {} …", v, crate::sdk::SDK_VERSION, sdk_root.display());
            } else {
                println!("📦 Installing SDK {} at {} …", crate::sdk::SDK_VERSION, sdk_root.display());
            }
            crate::sdk::export_to(&sdk_root, true)?;
        }
    }

    println!("\n🎉 Installation complete! You can now run 'lex' from any terminal.");
    Ok(())
}

/// Normalize one PATH entry for comparison. Windows: unify separators,
/// drop trailing slashes, lowercase (case-insensitive FS). Unix/macOS:
/// only drop trailing slashes — entries are case-SENSITIVE there, so
/// lowercasing would fake matches (`~/Bin` ≠ `~/bin`). Pure function.
pub fn normalize_path_entry(entry: &str) -> String {
    let mut s = entry.trim().to_string();
    if cfg!(windows) {
        s = s.replace('/', "\\").to_lowercase();
    }
    while s.len() > 1 && (s.ends_with('\\') || s.ends_with('/')) {
        s.pop();
    }
    s
}

/// True if `dir` is already listed in the platform-separated `path_var`.
/// Pure function — unit tested.
pub fn path_contains_entry(path_var: &str, dir: &Path) -> bool {
    let want = normalize_path_entry(&dir.to_string_lossy());
    let sep = if cfg!(windows) { ';' } else { ':' };
    path_var
        .split(sep)
        .map(normalize_path_entry)
        .any(|p| p == want)
}

fn is_in_path(dir: &Path) -> bool {
    // 1. Current process environment (fast path; fresh terminals see it).
    if let Ok(path_env) = env::var("PATH") {
        if path_contains_entry(&path_env, dir) {
            return true;
        }
    }
    // 2. Persistent store. The process environment can be STALE (entries
    // added after the terminal started never appear here), which used
    // to cause duplicate appends on every single `lex` invocation.
    if cfg!(windows) {
        if let Some(stored) = read_user_path_registry() {
            if path_contains_entry(&stored, dir) {
                return true;
            }
        }
    }
    false
}

/// Read the persistent per-user PATH from the registry without
/// powershell (fast, no extra process startup cost on every run).
fn read_user_path_registry() -> Option<String> {
    let out = Command::new("reg")
        .args(["query", r"HKCU\Environment", "/v", "Path"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        // `    Path    REG_EXPAND_SZ    <value...>`
        if let Some(idx) = line.find("REG_EXPAND_SZ") {
            return Some(line[idx + "REG_EXPAND_SZ".len()..].trim().to_string());
        }
        if let Some(idx) = line.find("REG_SZ") {
            return Some(line[idx + "REG_SZ".len()..].trim().to_string());
        }
    }
    None
}

/// Copy a toolchain binary over its destination, surviving the classic
/// self-update hazards on every OS:
/// - Unix `ETXTBSY`: the kernel refuses writes to a *running* executable.
///   Fall back to write-aside + atomic rename (inode swap), which the
///   kernel allows while the old image keeps running.
/// - Windows sharing violation on a running image: both attempts fail and
///   the error surfaces (auto-install ignores it silently; a fresh process
///   retries the update on next launch).
/// Always restores the owner-execute bit on Unix (`fs::copy` alone may
/// drop it, yielding "permission denied" on first run).
pub(crate) fn copy_exe_overwrite(src: &Path, dest: &Path) -> Result<()> {
    if fs::copy(src, dest).is_ok() {
        make_executable(dest)?;
        return Ok(());
    }
    let tmp = dest.with_extension("new");
    fs::copy(src, &tmp).with_context(|| format!("copying {} -> {}", src.display(), tmp.display()))?;
    make_executable(&tmp)?;
    fs::rename(&tmp, dest).with_context(|| format!("installing {}", dest.display()))?;
    Ok(())
}

/// `fs::copy` is not guaranteed to preserve the owner-execute bit on
/// Unix/macOS (fresh files get `0666 & ~umask`). Without this, the
/// installed `lex` fails with "permission denied" on first run.
#[cfg(unix)]
pub(crate) fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

fn add_to_path(dir: &Path) -> Result<()> {
    let dir_str = dir.to_string_lossy();

    if cfg!(windows) {
        // Re-read the registry value and skip when already present:
        // double-checked here so concurrent `lex` invocations can never
        // stack duplicates.
        if let Some(stored) = read_user_path_registry() {
            if path_contains_entry(&stored, dir) {
                return Ok(());
            }
            let sep = if stored.trim_end().ends_with(';') || stored.is_empty() {
                ""
            } else {
                ";"
            };
            let script = format!(
                "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + '{}{}', 'User')",
                sep, dir_str
            );
            Command::new("powershell")
                .args(["-NoProfile", "-Command", &script])
                .output()?;
            return Ok(());
        }
        // Fallback when the registry cannot be read.
        let script = format!(
            "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';{}', 'User')",
            dir_str
        );
        Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()?;
    } else {
        // Unix/macOS: append to POSIX login/interactive configs. macOS
        // Terminal.app runs login shells (`.zprofile`, not `.bashrc`), so
        // both must be covered; Linux leans on `.bashrc`/`.profile`.
        // Fish uses its own syntax in `config.fish`.
        let home = env::var("HOME")?;
        let export_line = format!("\nexport PATH=\"$PATH:{}\"\n", dir_str);
        let fish_line = format!("\nset -gx PATH {} $PATH\n", dir_str);

        let shell_configs = [".bashrc", ".zshrc", ".profile", ".zprofile", ".bash_profile"];
        for config in shell_configs {
            let config_path = PathBuf::from(&home).join(config);
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)?;
                if !content.contains(&*dir_str) {
                    let mut file = fs::OpenOptions::new().append(true).open(&config_path)?;
                    use std::io::Write;
                    file.write_all(export_line.as_bytes())?;
                }
            }
        }
        let fish_path = PathBuf::from(&home).join(".config").join("fish").join("config.fish");
        if fish_path.exists() {
            let content = fs::read_to_string(&fish_path)?;
            if !content.contains(&*dir_str) {
                let mut file = fs::OpenOptions::new().append(true).open(&fish_path)?;
                use std::io::Write;
                file.write_all(fish_line.as_bytes())?;
            }
        }
    }
    Ok(())
}

/// Exact lines written by [`add_to_path`] on Unix (used by `uninstall`
/// to remove only what we added — never user content).
#[cfg(unix)]
fn unix_added_lines(dir: &Path) -> Vec<String> {
    let dir_str = dir.to_string_lossy();
    vec![
        format!("export PATH=\"$PATH:{}\"", dir_str),
        format!("set -gx PATH {} $PATH", dir_str),
    ]
}

/// Removes our own PATH lines from Unix shell configs (best-effort).
#[cfg(unix)]
fn remove_from_shell_configs(dir: &Path) {
    let Ok(home) = env::var("HOME") else { return };
    let markers = unix_added_lines(dir);
    let mut configs = vec![".bashrc", ".zshrc", ".profile", ".zprofile", ".bash_profile"]
        .into_iter()
        .map(|c| PathBuf::from(&home).join(c))
        .collect::<Vec<_>>();
    configs.push(PathBuf::from(&home).join(".config").join("fish").join("config.fish"));
    for path in configs {
        let Ok(content) = fs::read_to_string(&path) else { continue };
        let kept: Vec<&str> = content
            .lines()
            .filter(|l| !markers.iter().any(|m| l.trim() == m.trim()))
            .collect();
        if kept.len() != content.lines().count() {
            let _ = fs::write(&path, kept.join("\n") + "\n");
        }
    }
}

pub fn uninstall() -> Result<()> {
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| anyhow!("Could not find home directory"))?;
    
    let lexicon_home = PathBuf::from(home).join(".lexicon");
    let bin_dir = lexicon_home.join("bin");
    let exe_name = if cfg!(windows) { "lex.exe" } else { "lex" };
    let target_exe = bin_dir.join(exe_name);
    
    if target_exe.exists() {
        println!("🗑️  Removing Lexicon binary from {:?}", target_exe);
        fs::remove_file(&target_exe)?;
        println!("✅ Binary removed successfully!");
    } else {
        println!("⚠️  Lexicon binary not found at {:?}", target_exe);
    }

    // Installed SDK tree (managed by first-run auto-install / install).
    let sdk_dir = lexicon_home.join("sdk");
    if sdk_dir.exists() {
        println!("🗑️  Removing installed SDK at {:?}", sdk_dir);
        fs::remove_dir_all(&sdk_dir)?;
    }

    // Unix: remove only the PATH lines we added (best-effort).
    #[cfg(unix)]
    remove_from_shell_configs(&bin_dir);
    // Check if bin directory is empty
    if bin_dir.exists() {
        if let Ok(entries) = fs::read_dir(&bin_dir) {
            if entries.count() == 0 {
                println!("🗑️  Removing empty bin directory: {:?}", bin_dir);
                fs::remove_dir(&bin_dir)?;
            }
        }
    }
    
    // Check if lexicon home is empty
    if lexicon_home.exists() {
        if let Ok(entries) = fs::read_dir(&lexicon_home) {
            if entries.count() == 0 {
                println!("🗑️  Removing .lexicon directory: {:?}", lexicon_home);
                fs::remove_dir(&lexicon_home)?;
            }
        }
    }
    
    println!("\n✨ Lexicon has been uninstalled!");
    println!("💡 Note: You may need to restart your terminal or manually remove .lexicon from your PATH.");
    
    Ok(())
}

#[cfg(test)]
mod installer_tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn normalize_entry() {
        assert_eq!(normalize_path_entry("C:\\A\\b\\"), "c:\\a\\b");
        assert_eq!(normalize_path_entry("C:/A/b/"), "c:\\a\\b");
        assert_eq!(normalize_path_entry("  C:\\A  "), "c:\\a");
    }

    #[test]
    #[cfg(not(windows))]
    fn normalize_entry_unix_keeps_case() {
        // Unix/macOS filesystems are case-sensitive: never lowercase.
        assert_eq!(normalize_path_entry("/Home/A/b/"), "/Home/A/b");
        assert_eq!(normalize_path_entry("  /x  "), "/x");
    }

    #[test]
    #[cfg(windows)]
    fn detects_present_entry_case_insensitively() {
        let dir = Path::new(r"C:\Users\Admin\.lexicon\bin");
        assert!(path_contains_entry(
            r"C:\Windows;C:\USERS\ADMIN\.LEXICON\BIN;C:\Tools",
            dir
        ));
    }

    #[test]
    #[cfg(not(windows))]
    fn detects_present_entry_case_sensitively() {
        let dir = Path::new("/home/u/.lexicon/bin");
        assert!(path_contains_entry("/usr/bin:/home/u/.lexicon/bin", dir));
        assert!(!path_contains_entry("/usr/bin:/home/u/.LEXICON/BIN", dir));
    }

    #[test]
    fn detects_present_entry_with_trailing_slash() {
        let dir = Path::new(r"C:\Users\Admin\.lexicon\bin");
        assert!(path_contains_entry(
            r"C:\Windows;C:\Users\Admin\.lexicon\bin\;C:\Tools",
            dir
        ));
    }

    #[test]
    fn rejects_absent_and_partial_matches() {
        let dir = Path::new(r"C:\Users\Admin\.lexicon\bin");
        assert!(!path_contains_entry(r"C:\Windows;C:\Tools", dir));
        // Prefix alone must not match: entry must be complete.
        assert!(!path_contains_entry(r"C:\Users\Admin\.lexicon", dir));
    }

    #[test]
    fn empty_path_var_is_absent() {
        let dir = Path::new(r"C:\Users\Admin\.lexicon\bin");
        assert!(!path_contains_entry("", dir));
    }
}
