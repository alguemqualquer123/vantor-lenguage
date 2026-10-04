use std::path::PathBuf;

fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=uuid");
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=oleaut32");
        println!("cargo:rustc-link-lib=shlwapi");
        println!("cargo:rustc-link-lib=shell32");
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=gdi32");
    }
    // Release auto-version: every release compile bumps the workspace
    // patch version (0.3.1 → 0.3.2 → …), so `lex version`, fingerprints,
    // SBOMs and the installed-SDK drift check always move forward.
    // Debug builds never touch versions (reproducible inner loop).
    // Opt out with LEX_NO_AUTO_BUMP=1 (locked CI or reproducible builds).
    if std::env::var("PROFILE").as_deref() == Ok("release")
        && std::env::var("LEX_NO_AUTO_BUMP").is_err()
    {
        auto_bump_patch();
    }
}

/// Increment `version = "x.y.z"` under `[workspace.package]` in the
/// workspace root `Cargo.toml`. Idempotent per distinct build (each
/// release compile moves exactly one patch step).
fn auto_bump_patch() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest_dir.join("..").join("..").join("Cargo.toml");
    let Ok(content) = std::fs::read_to_string(&root) else {
        return;
    };
    let mut in_workspace_package = false;
    let mut out = String::with_capacity(content.len() + 8);
    let mut bumped = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_workspace_package = trimmed == "[workspace.package]";
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_workspace_package && trimmed.starts_with("version") && bumped.is_empty() {
            if let Some(next) = bump_line(line) {
                bumped = next.clone();
                out.push_str(&next);
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    if bumped.is_empty() {
        return;
    }
    if std::fs::write(&root, out).is_ok() {
        println!(
            "cargo::warning=release build: toolchain version bumped to {} (LEX_NO_AUTO_BUMP=1 to lock)",
            bumped.trim()
        );
    }
}

/// `"version = \"0.3.1\""` → `"version = \"0.3.2\""` (patch +1, quotes and
/// spacing preserved). Returns `None` when the line is not a plain
/// `version = "x.y.z"` assignment.
fn bump_line(line: &str) -> Option<String> {
    let (head, tail) = line.split_once('=')?;
    if head.trim() != "version" {
        return None;
    }
    let v = tail.trim().trim_matches('"');
    let mut parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    let patch: u64 = parts[2].parse().ok()?;
    Some(format!(
        "{}version = \"{}.{}.{}\"",
        head_before_eq(line),
        parts[0],
        parts[1],
        patch + 1
    ))
}

/// Whitespace before `=` (keeps alignment of the rewritten line).
fn head_before_eq(line: &str) -> String {
    match line.find('=') {
        Some(i) => line[..i].to_string(),
        None => String::new(),
    }
}