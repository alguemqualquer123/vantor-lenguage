use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, anyhow, Context};

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

    // 2. Copy current executable to bin directory
    let mut target_exe = bin_dir.join(exe_name);
    if cfg!(windows) {
        target_exe.set_extension("exe");
    }

    // Only copy if it's different or doesn't exist
    if !target_exe.exists() || fs::metadata(&current_exe)?.len() != fs::metadata(&target_exe).map(|m| m.len()).unwrap_or(0) {
        fs::copy(&current_exe, &target_exe)?;
        println!("🚀 Lexicon binary installed to {:?}", target_exe);
    }

    // 3. Add to PATH if not already there
    if !is_in_path(&bin_dir) {
        add_to_path(&bin_dir)?;
        println!("✅ Lexicon added to system PATH. You may need to restart your terminal.");
    }

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
    fs::copy(&source_exe, &target_exe)?;

    // 4. Update PATH
    if !is_in_path(&bin_dir) {
        add_to_path(&bin_dir)?;
        println!("✅ Lexicon added to system PATH successfully!");
    } else {
        println!("✨ Lexicon is already in your system PATH.");
    }

    println!("\n🎉 Installation complete! You can now run 'lex' from any terminal.");
    Ok(())
}

fn is_in_path(dir: &Path) -> bool {
    if let Ok(path_env) = env::var("PATH") {
        let dir_str = dir.to_string_lossy().to_lowercase();
        return path_env.split(if cfg!(windows) { ';' } else { ':' })
            .any(|p| p.to_lowercase() == dir_str);
    }
    false
}

fn add_to_path(dir: &Path) -> Result<()> {
    let dir_str = dir.to_string_lossy();

    if cfg!(windows) {
        // Use PowerShell to add to User PATH persistently
        let script = format!(
            "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';{}', 'User')",
            dir_str
        );
        Command::new("powershell")
            .args(["-Command", &script])
            .output()?;
    } else {
        // Unix: Add to .bashrc and .zshrc
        let home = env::var("HOME")?;
        let shell_configs = [".bashrc", ".zshrc", ".profile"];
        let export_line = format!("\nexport PATH=\"$PATH:{}\"\n", dir_str);

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
    }
    Ok(())
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
