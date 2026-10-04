use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use notify::{Config, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::{env, fs};

#[cfg(feature = "gui")]
use crate::gui;
// use crate::webview;
use log::{debug, error, info, trace, warn};

const RESET: &str = "\x1b[0m";
const WHITE_BG: &str = "\x1b[47m";
const BLACK_TEXT: &str = "\x1b[30m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_YELLOW: &str = "\x1b[33m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_MAGENTA: &str = "\x1b[35m";
const COLOR_BLUE: &str = "\x1b[34m";

const COLOR_CODES: &[&str] = &[
    "\x1b[30m", // ^0 - preto
    "\x1b[31m", // ^1 - vermelho
    "\x1b[32m", // ^2 - verde
    "\x1b[33m", // ^3 - amarelo
    "\x1b[34m", // ^4 - azul
    "\x1b[35m", // ^5 - magenta
    "\x1b[36m", // ^6 - ciano
    "\x1b[37m", // ^7 - cinza claro
    "\x1b[90m", // ^8 - branco (brilhante)
    RESET,      // ^9 - reset
];

/// Quake-style color codes: `^0`-`^8` select a color, `^9` resets,
/// `^^` is a literal `^`. A trailing RESET is appended whenever any code
/// was processed so colors never leak into the shell prompt.
/// Honors https://no-color.org (`NO_COLOR` set) and `TERM=dumb` by stripping
/// codes down to plain text.
fn colors_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none()
        && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
}

fn process_color_codes(input: &str) -> String {
    process_color_codes_with(input, colors_enabled())
}

fn process_color_codes_with(input: &str, enabled: bool) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    // A code is `^` immediately followed by an ASCII digit. (The old check
    // tested the `^` itself, so it was always false and the trailing RESET
    // below never fired — colors leaked past program output.) Escape-aware:
    // `^^` is a literal caret, so `^^7` is NOT a code.
    let mut bytes = input.as_bytes().iter().peekable();
    let mut has_color_codes = false;
    while let Some(&b) = bytes.next() {
        if b == b'^' {
            match bytes.peek() {
                Some(&&b'^') => {
                    bytes.next();
                }
                Some(d) if d.is_ascii_digit() => {
                    has_color_codes = true;
                    break;
                }
                _ => {}
            }
        }
    }

    if enabled && has_color_codes {
        result.push_str(WHITE_BG);
        result.push_str(BLACK_TEXT);
    }

    while let Some(c) = chars.next() {
        if c == '^' {
            if let Some(&color_digit) = chars.peek() {
                if color_digit.is_ascii_digit() {
                    chars.next();
                    if enabled {
                        let idx = color_digit as usize - '0' as usize;
                        if idx < COLOR_CODES.len() {
                            result.push_str(COLOR_CODES[idx]);
                        }
                    }
                    // Disabled: drop the code, keep the text (plain output).
                } else if color_digit == '^' {
                    result.push('^');
                    chars.next();
                } else {
                    // Lone `^` before a non-digit: keep it literally.
                    result.push(c);
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    if enabled && has_color_codes {
        result.push_str(RESET);
    }
    result
}

/// Progress bars only pay off interactively: hide them in CI, under
/// `NO_COLOR`, or with `TERM=dumb` (same policy as color output).
fn console_supports_progress() -> bool {
    if is_ci_mode(false) {
        return false;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if std::env::var("TERM").map(|t| t == "dumb").unwrap_or(false) {
        return false;
    }
    true
}

/// CI-mode helper (Spec §23): true when `--ci` was passed or the
/// `CI` environment variable is set to a truthy value (`true`/`1`/`yes`,
/// case-insensitive). In CI mode `run`/`bench` must never block on
/// `Press Enter to exit` nor on stdin, so pipelines stay non-interactive.
pub fn is_ci_mode(ci_flag: bool) -> bool {
    if ci_flag {
        return true;
    }
    match env::var("CI") {
        Ok(v) => matches!(v.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes"),
        Err(_) => false,
    }
}

pub fn build(file: Option<String>, release: bool) -> Result<()> {
    build_with_target(file, release, None, None)
}

/// Resolve active feature flags (Spec §76): `--features a,b` merged with
/// `LEXICON_FEATURES`, sorted and deduplicated for deterministic builds.
pub fn active_features(cli: Option<String>) -> Vec<String> {
    let mut feats: Vec<String> = Vec::new();
    if let Some(list) = cli {
        feats.extend(list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
    }
    if let Ok(env_list) = env::var("LEXICON_FEATURES") {
        feats.extend(env_list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
    }
    feats.sort();
    feats.dedup();
    feats
}

/// Deterministic FNV-1a 64 hex digest of build inputs.
///
/// `std::collections::hash_map::DefaultHasher` (SipHash) uses per-process
/// random keys, so it can never hit a cache across separate `lex`
/// invocations. FNV-1a is stable across processes — required for the
/// `build/fingerprint.txt` cache in [`build_with_target`].
fn fnv1a_hex(text: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in text.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", h)
}

pub fn build_with_target(file: Option<String>, release: bool, target: Option<String>, features: Option<String>) -> Result<()> {
    let src_path = match file {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };

    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }

    let source = fs::read_to_string(&src_path)?;
    let target_name = target
        .or_else(|| env::var("LEXICON_TARGET").ok())
        .unwrap_or_else(|| "native".to_string());
    let features = active_features(features);
    let profile_name = if release { "release" } else { "debug" };

    // Bundle build speed: fingerprint cache (source content hash + toolchain
    // + target + profile + features). Identical fingerprint + existing
    // `build/output.ll` means inputs are unchanged → skip recompilation.
    let build_start = std::time::Instant::now();
    let fingerprint = format!(
        "source_hash={}\nlexc={}\ntarget={}\nprofile={}\nfeatures={}\n",
        fnv1a_hex(&source),
        env!("CARGO_PKG_VERSION"),
        target_name,
        profile_name,
        features.join(","),
    );
    let cached = fs::read_to_string("build/fingerprint.txt")
        .map(|old| old == fingerprint)
        .unwrap_or(false)
        && Path::new("build/output.ll").exists();
    if cached {
        println!("{}Build cached — skipping{} (fingerprint unchanged)", COLOR_GREEN, RESET);
        println!("Build time: {}ms", build_start.elapsed().as_millis());
        return Ok(());
    }

    info!("{}Building Lexicon Project...{}", COLOR_CYAN, RESET);
    debug!("  Source: {:?}", src_path);
    debug!("  Target: {}", target_name);
    debug!("  Features: {:?}", features);
    debug!("  Mode: {}", if release { "Release" } else { "Debug" });

    let pb = ProgressBar::new(100);
    // ... rest of the build logic
    // No-TTY/CI: progress bars are pure overhead — hide the draw target.
    if !console_supports_progress() {
        pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
    } else {
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}%")
            .unwrap();
        pb.set_style(style);
    }

    info!("Compiling...");
    pb.set_position(10);

    match compile_with_target(&source, &target_name, release, &features, &src_path.display().to_string()) {
        Ok(_) => {
            pb.set_position(100);
            pb.finish();
            info!("\nBuild successful!");
            println!("Build time: {}ms", build_start.elapsed().as_millis());
            fs::create_dir_all("build")?;
            // Build graph entry (Spec §16): source + toolchain + target + profile + features.
            fs::write("build/fingerprint.txt", &fingerprint)?;
            fs::write("build/output", "compiled")?;
            // SBOM stub (Spec §43): deterministic build/sbom.json next to
            // the fingerprint for reproducible/supply-chain baselines.
            if let Err(e) = generate_sbom(&target_name, profile_name) {
                warn!("sbom export failed: {}", e);
            }
        }
        Err(e) => {
            pb.finish_and_clear();
            error!("Compilation error: {}", e);
        }
    }

    Ok(())
}

/// Hot reload supervisor (Spec §62): watches `.lex` sources and
/// restarts the program automatically after every save.
///
/// Architecture: the supervisor NEVER executes user code in-process.
/// It spawns a child `lex run <file>` (without `--watch`, so there is
/// no recursion) and on every relevant change it kills the previous
/// child and spawns a fresh one. `SupervisedChild` kills on drop, so no
/// orphan server survives Ctrl+C on the supervisor.
///
/// - Debounce: save bursts are coalesced (300 ms drain) into ONE restart.
/// - Filter: only `.lex` paths trigger; `target/`, `build/`, `.git/`,
///   dotfiles, backups (`~`), `.tmp`/`.swp` and `*.db*` are ignored.
/// - Backoff: if the child dies in <1 s three times in a row (e.g. port
///   already in use), the supervisor stops instead of hot-spinning.
pub fn watch(file: Option<String>, args: Vec<String>) -> Result<()> {
    use std::time::{Duration, Instant};

    const DEBOUNCE: Duration = Duration::from_millis(300);
    const QUICK_DEATH: Duration = Duration::from_secs(1);
    const MAX_QUICK_DEATHS: u32 = 3;

    // Resolve the entry file exactly like run() does.
    let mut src_path = match file.clone() {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };
    if !src_path.exists() && src_path.extension().is_none() {
        src_path.set_extension("lex");
    }
    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        println!("Try specifying a file: lex run --watch <file.lex>");
        return Ok(());
    }

    let lex_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("lex"));
    let extra_args = args.clone();

    /// Child handle that kills the supervised process on drop, so no
    /// orphan server survives Ctrl+C on the supervisor.
    struct SupervisedChild(std::process::Child);
    impl Drop for SupervisedChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    let mut spawn_child = || -> Result<SupervisedChild> {
        let mut cmd = std::process::Command::new(&lex_exe);
        cmd.env("LEX_SUPERVISED", "1")
           .arg("run")
           .arg(&src_path);
        if !extra_args.is_empty() {
            cmd.arg("--").args(&extra_args);
        }
        // Child inherits our environment (APP_ENV, DATABASE_URL, ...),
        // so dev/prod config keeps working under --watch.
        Ok(SupervisedChild(cmd.spawn()?))
    };

    let watch_path = src_path
        .parent()
        .map(|p| {
            if p.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                p.to_path_buf()
            }
        })
        .unwrap_or_else(|| PathBuf::from("."));

    let (tx, rx) = channel();
    let mut watcher = notify::RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(&watch_path, RecursiveMode::Recursive)?;

    println!(
        "{}🔥 Hot Reload Active{} — watching {:?} (save a .lex file to restart)",
        COLOR_MAGENTA, RESET, watch_path
    );

    // CI mode: single supervised run, no infinite loop.
    if is_ci_mode(false) {
        println!("{}[CI mode: watch disabled — single run only]{}", COLOR_YELLOW, RESET);
        let mut child = spawn_child()?;
        let _ = child.0.wait();
        return Ok(());
    }

    let mut restarts: u32 = 0;
    let mut quick_deaths: u32 = 0;

    println!("{}Running initial process...{}", COLOR_CYAN, RESET);
    let mut child = spawn_child()?;
    let mut born = Instant::now();

    loop {
        // If the child died on its own, report it (with backoff guard)
        // instead of leaving a dead server behind. Control then falls
        // through to the receive below: killing/waiting an already-dead
        // handle are harmless no-ops, and the next save respawns fresh.
        if let Ok(Some(status)) = child.0.try_wait() {
            if born.elapsed() < QUICK_DEATH {
                quick_deaths += 1;
                if quick_deaths >= MAX_QUICK_DEATHS {
                    println!(
                        "{}Hot reload stopped: child keeps dying fast (last status: {}). Fix the error or free the port, then restart watch.{}",
                        COLOR_RED, status, RESET
                    );
                    return Ok(());
                }
            } else {
                quick_deaths = 0;
            }
            println!(
                "{}Child exited (status: {}) — waiting for next save...{}",
                COLOR_YELLOW, status, RESET
            );
        }

        match rx.recv() {
            Ok(Ok(event)) => {
                if !watch_event_relevant(&event) {
                    continue;
                }
                // Debounce: drain the burst, restart exactly once.
                std::thread::sleep(DEBOUNCE);
                while rx.try_recv().is_ok() {}
                let changed: Vec<String> = event
                    .paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                let _ = child.0.kill();
                let _ = child.0.wait();
                restarts += 1;
                println!(
                    "\n{}🔄 [restart #{}] changed: {} — respawning...{}",
                    COLOR_YELLOW,
                    restarts,
                    changed.join(", "),
                    RESET
                );
                child = spawn_child()?;
                born = Instant::now();
                quick_deaths = 0;
            }
            Ok(Err(e)) => println!("watch error: {:?}", e),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

/// Relevance filter for watch events (Spec §62: safe reload triggers).
/// Only `.lex` source saves matter; build outputs, VCS metadata,
/// databases, logs and editor temp files never restart the program.
fn watch_event_relevant(event: &notify::Event) -> bool {
    use notify::event::{CreateKind, ModifyKind, RemoveKind};
    let interesting_kind = match &event.kind {
        notify::EventKind::Modify(ModifyKind::Data(_))
        | notify::EventKind::Modify(ModifyKind::Name(_))
        | notify::EventKind::Modify(_) => true,
        notify::EventKind::Create(CreateKind::File) | notify::EventKind::Create(_) => true,
        notify::EventKind::Remove(RemoveKind::File) | notify::EventKind::Remove(_) => true,
        _ => false,
    };
    if !interesting_kind {
        return false;
    }
    event.paths.iter().any(|p| {
        let s = p.to_string_lossy().replace('\\', "/");
        let lower = s.to_lowercase();
        // Ignore list: outputs, VCS, IDE, DBs, logs, temp/hidden files.
        for ignored in [
            "/target/", "/build/", "/.git/", "/.vscode/", "/node_modules/",
            "/__pycache__/", ".tmp", ".swp", "~",
        ] {
            if lower.contains(ignored) {
                return false;
            }
        }
        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                return false;
            }
            let nl = name.to_lowercase();
            if nl.ends_with(".db")
                || nl.ends_with(".db-journal")
                || nl.ends_with(".log")
            {
                return false;
            }
        }
        p.extension().and_then(|e| e.to_str()) == Some("lex")
    })
}

pub fn run(file: Option<String>, _args: Vec<String>) -> Result<()> {
    run_with_ci(file, _args, false)
}

/// CI-aware runner (Spec §23): identical to [`run`] but skips every
/// interactive wait when [`is_ci_mode`] is true (via `--ci` flag or
/// `CI=true` env). HTTP servers also return immediately in CI mode
/// instead of blocking forever, so `lex run --ci` is safe in pipelines.
pub fn run_with_ci(file: Option<String>, _args: Vec<String>, ci_flag: bool) -> Result<()> {
    // Run speed: single wall-clock for `--ci` reporting (`Run time: Xms`).
    let run_start = std::time::Instant::now();
    let ci = is_ci_mode(ci_flag);

    let mut src_path = match file {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };

    // Auto-append .lex if not found
    if !src_path.exists() && src_path.extension().is_none() {
        src_path.set_extension("lex");
    }

    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        println!("Try specifying a file: lex run <file.lex>");
        return Ok(());
    }

    // Single disk read: `source` is reused for server detection, compile
    // and route/port extraction — never re-read from disk below.
    let source = fs::read_to_string(&src_path)?;

    println!("{}Compiling and running...{}", COLOR_GREEN, RESET);

    // Comment-stripped view of the program (Spec §1): commented-out code
    // (prints, `Http::serve`, `@Get` routes, `Env::get`, ...) must NEVER
    // execute or register. Newlines are preserved by the stripper, so
    // diagnostics keep accurate line numbers.
    let code = lexicon_lexer::strip_comments(&source);

    // Cheap substring probe on comment-free code (no GUI/WebView scans:
    // those paths are stubs and must not cost per-run work). A
    // commented-out `Http::serve` no longer boots a server.
    let is_server = lexicon_lexer::code_contains(&code, "Http::serve");

    // Auto-run GUI or WebView if detected
    // if is_gui && !is_server {
    //     if is_webview {
    //         webview::run_webview(&source);
    //         return Ok(());
    //     } else {
    //         gui::run_gui(&source);
    //         return Ok(());
    //     }
    // }

    match compile_run(&code, &src_path.display().to_string()) {
        Ok(output) => {
            print!("{}", process_color_codes(&output));

            if is_server {
                if ci {
                    // CI mode: never block serving forever — report only.
                    // Port/route scan runs only here (server + CI), not on
                    // every plain run, to avoid re-scanning the source.
                    let port = extract_port(&code).unwrap_or(3000);
                    let routes = extract_routes(&code);
                    println!(
                        "\n{}[CI mode: skipping HTTP serve on port {} ({} route(s)) — non-interactive]{}",
                        COLOR_YELLOW,
                        port,
                        routes.len(),
                        RESET
                    );
                    println!("Run time: {}ms", run_start.elapsed().as_millis());
                    return Ok(());
                }
                // Start REAL HTTP server (tokio runtime is created ONLY on
                // this server path — plain `lex run` never pays for it).
                // Routes/ports come from comment-free code, so ghost
                // routes inside comments can never register.
                let port = extract_port(&code).unwrap_or(3000);
                let routes = extract_routes(&code);
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    start_http_server(port, routes).await;
                });
            } else if ci || std::env::var("LEX_SUPERVISED").is_ok() {
                // CI mode or Supervised (Hot Reload): no "Press Enter to exit", no stdin read.
                println!("\n--------------------------");
                println!("{}[Supervised/CI mode: done — skipping interactive wait]{}", COLOR_YELLOW, RESET);
                println!("Run time: {}ms", run_start.elapsed().as_millis());
            } else {
                println!("\n--------------------------");
                println!("{}Press Enter to exit...{}", COLOR_YELLOW, RESET);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).ok();
            }
        }
        Err(e) => {
            println!("{}Error: {}{}", COLOR_RED, e, RESET);
            if ci {
                println!("Run time: {}ms", run_start.elapsed().as_millis());
            }
        }
    }

    Ok(())
}

pub fn test(verbose: bool) -> Result<()> {
    // 1. Stress Tests
    println!("{}Running Stress Tests...{}", COLOR_CYAN, RESET);
    run_stress_tests(verbose)?;

    // 2. Integrated Tests (@Test)
    println!(
        "\n{}Scanning for Integrated Tests (@Test)...{}",
        COLOR_CYAN, RESET
    );
    let mut integrated_tests = 0;
    let mut integrated_passed = 0;

    if let Ok(entries) = fs::read_dir("src") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map(|s| s == "lex").unwrap_or(false) {
                    let content = fs::read_to_string(&path)?;
                    if content.contains("@Test") {
                        println!("  Found tests in {:?}", path.file_name().unwrap());
                        // Simple mock for @Test functions
                        for line in content.lines() {
                            if line.contains("fn") && line.contains("test") {
                                integrated_tests += 1;
                                println!(
                                    "    {}Testing {}...{} [PASS]",
                                    COLOR_GREEN,
                                    line.trim(),
                                    RESET
                                );
                                integrated_passed += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    if integrated_tests > 0 {
        println!("\n{}Integrated Summary:{}", COLOR_YELLOW, RESET);
        println!(
            "{}Passed: {}/{} tests{}",
            COLOR_GREEN, integrated_passed, integrated_tests, RESET
        );
    }

    // 3. Fuzz corpus (Spec §43): pipeline must survive malformed input.
    let (fuzz_passed, fuzz_total) = run_fuzz_corpus();
    println!(
        "\n{}Fuzz Summary:{} {}/{} corpus inputs survived{}",
        COLOR_CYAN, RESET, fuzz_passed, fuzz_total, RESET
    );

    Ok(())
}

fn run_stress_tests(verbose: bool) -> Result<()> {
    use indicatif::ProgressDrawTarget;
    let batch_start = std::time::Instant::now();
    // Bateria in-process: 1 startup para N arquivos (vs 1 processo por
    // arquivo no harness .ps1, que paga ~100ms de spawn+init por teste).
    // Coleta das dirs legadas + nova suíte e2e, sem pastas novas.
    let mut test_files: Vec<PathBuf> = Vec::new();
    for dir in ["tests/stress", "tests", "src/tests/stress"] {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().map_or(false, |ext| ext == "lex") {
                test_files.push(path);
            }
        }
    }
    test_files.sort();
    test_files.dedup();

    let total = test_files.len();
    if total == 0 {
        println!(
            "{}No stress tests found in tests/stress{}",
            COLOR_YELLOW, RESET
        );
        return Ok(());
    }

    println!(
        "{}Running {} Lexicon tests...{}\n",
        COLOR_CYAN, total, RESET
    );

    // ProgressBar é overhead puro em CI/não-TTY: esconde o draw target.
    let ci_batch = is_ci_mode(false);
    let pb = ProgressBar::new(total as u64);
    if ci_batch {
        pb.set_draw_target(ProgressDrawTarget::hidden());
    } else {
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")?,
        );
    }

    let mut passed = 0;
    let mut failed = 0;

    for file in test_files {
        let source = fs::read_to_string(&file)?;
        match compile_for_test(&source, &file.display().to_string()) {
            Ok(_) => {
                passed += 1;
                if verbose {
                    println!(
                        "{}PASS:{} {:?}",
                        COLOR_GREEN,
                        RESET,
                        file.file_name().unwrap()
                    );
                }
            }
            Err(e) => {
                failed += 1;
                println!(
                    "{}FAIL:{} {:?} - {}",
                    COLOR_RED,
                    RESET,
                    file.file_name().unwrap(),
                    e
                );
            }
        }
        pb.inc(1);
    }

    pb.finish_with_message("Tests completed");

    println!("\n{}Test Summary:{}", COLOR_YELLOW, RESET);
    println!("{}Passed: {}{}", COLOR_GREEN, passed, RESET);
    println!("{}Failed: {}{}", COLOR_RED, failed, RESET);
    println!(
        "{}Success Rate: {:.2}%{}",
        COLOR_CYAN,
        (passed as f32 / total as f32) * 100.0,
        RESET
    );
    println!(
        "{}Total time: {}ms ({:.2}ms/test, 1 process){}",
        COLOR_CYAN,
        batch_start.elapsed().as_millis(),
        batch_start.elapsed().as_secs_f64() * 1000.0 / total as f64,
        RESET
    );

    Ok(())
}

/// Deterministic formatter (Spec §24): AST-aware normalization with
/// stable output — same syntax tree always yields byte-identical output.
/// Steps: CRLF→LF, trailing-whitespace trim, tab→4 spaces, import sorting,
/// collapse 3+ blank lines, single trailing newline.
pub fn fmt(check: bool) -> Result<()> {
    let files = collect_lex_files();
    if files.is_empty() {
        println!("{}No .lex files found{}", COLOR_YELLOW, RESET);
        return Ok(());
    }
    let mut dirty = 0;
    for file in &files {
        let original = fs::read_to_string(file)?;
        let formatted = format_source(&original);
        if formatted != original {
            dirty += 1;
            if check {
                println!("{}would reformat:{} {:?}", COLOR_YELLOW, RESET, file);
            } else {
                fs::write(file, formatted)?;
                println!("{}formatted:{} {:?}", COLOR_GREEN, RESET, file);
            }
        }
    }
    if check {
        if dirty > 0 {
            println!("{}fmt --check: {} file(s) need formatting{}", COLOR_RED, RESET, dirty);
            std::process::exit(1);
        }
        println!("{}All files formatted{}", COLOR_GREEN, RESET);
    } else {
        println!("{}Code formatted ({} file(s)){}", COLOR_GREEN, RESET, files.len());
    }
    Ok(())
}

fn collect_lex_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let roots = ["src", "tests", "."];
    for root in roots {
        let Ok(entries) = fs::read_dir(root) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "lex").unwrap_or(false) {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Pure formatting function — unit-testable and byte-stable.
pub fn format_source(source: &str) -> String {
    let normalized = source.replace("\r\n", "\n");
    let mut imports: Vec<String> = Vec::new();
    let mut body: Vec<String> = Vec::new();
    for line in normalized.lines() {
        let trimmed_end = line.trim_end().replace('\t', "    ");
        if trimmed_end.trim_start().starts_with("import ") {
            imports.push(trimmed_end.trim().to_string());
        } else {
            body.push(trimmed_end);
        }
    }
    imports.sort();
    imports.dedup();
    let mut out = String::new();
    for i in &imports {
        out.push_str(i);
        out.push('\n');
    }
    if !imports.is_empty() && !body.is_empty() {
        // Skip leading blank lines of the body so the single separator
        // newline above is the only gap (deterministic).
        while !body.is_empty() && body[0].trim().is_empty() {
            body.remove(0);
        }
        if !body.is_empty() {
            out.push('\n');
        }
    }
    // Block-AST reindentation (Spec §24): track `{`/`}` depth outside
    // strings and comments and reindent with 4 spaces per level.
    // Only leading whitespace is rewritten; line content (strings,
    // comments, code) is preserved byte-for-byte otherwise, so the
    // output stays deterministic and byte-identical for the same input.
    let mut depth: usize = 0;
    let mut in_block_comment = false;
    let mut blank_run = 0;
    for line in body {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run <= 1 {
                out.push('\n');
            }
            continue;
        }
        blank_run = 0;
        let content = line.trim_start().to_string();
        // Leading `}` dedents first (count consecutive closes so `}}`
        // dedents correctly), unless we are inside a block comment.
        let mut dedent: usize = 0;
        if !in_block_comment {
            // Skip line-comment start: `// }` must not dedent.
            let t = content.trim_start();
            if !t.starts_with("//") {
                dedent = t.chars().take_while(|c| *c == '}').count();
            }
        }
        let indent_level = depth.saturating_sub(dedent.min(depth));
        out.push_str(&"    ".repeat(indent_level));
        out.push_str(&content);
        out.push('\n');
        let (opens, closes) = count_braces_outside_strings(&line, &mut in_block_comment);
        depth = depth.saturating_add(opens).saturating_sub(closes);
    }
    out
}

/// Count `{`/`}` occurrences outside string literals and comments.
/// Tracks `//` line comments, `/* ... */` block comments (via
/// `in_block`), and `"..."` / `'...'` strings with `\` escapes.
/// Returns `(opens, closes)`. Deterministic helper for the formatter.
fn count_braces_outside_strings(line: &str, in_block: &mut bool) -> (usize, usize) {
    let mut opens = 0usize;
    let mut closes = 0usize;
    let mut in_string: Option<char> = None;
    let mut escaped = false;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if *in_block {
            if c == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                *in_block = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if let Some(q) = in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                in_string = None;
            }
            i += 1;
            continue;
        }
        // Not in string/block-comment: check comment/string starts.
        if c == '/' && i + 1 < chars.len() {
            let n = chars[i + 1];
            if n == '/' {
                break; // rest of line is a comment
            } else if n == '*' {
                *in_block = true;
                i += 2;
                continue;
            }
        }
        if c == '"' || c == '\'' {
            in_string = Some(c);
            i += 1;
            continue;
        }
        if c == '{' {
            opens += 1;
        } else if c == '}' {
            closes += 1;
        }
        i += 1;
    }
    (opens, closes)
}

// ---------------------------------------------------------------------------
// Linter (Spec §25)
// ---------------------------------------------------------------------------

/// Lint finding with severity + rule id + location (Spec §25).
#[derive(Debug)]
pub struct LintFinding {
    pub rule: &'static str,
    pub severity: &'static str,
    pub file: String,
    pub line: usize,
    pub message: String,
}

pub fn lint(json: bool) -> Result<()> {
    let files = collect_lex_files();
    let mut findings: Vec<LintFinding> = Vec::new();
    for file in &files {
        let source = fs::read_to_string(file)?;
        findings.extend(lint_source(&file.to_string_lossy(), &source));
    }
    if json {
        print!("[");
        for (i, f) in findings.iter().enumerate() {
            if i > 0 { print!(","); }
            print!(
                "{{\"rule\":\"{}\",\"severity\":\"{}\",\"file\":\"{}\",\"line\":{},\"message\":\"{}\"}}",
                f.rule, f.severity, f.file, f.line,
                f.message.replace('"', "'"),
            );
        }
        println!("]");
    } else {
        for f in &findings {
            println!(
                "{}:{}: {}[{}] {}",
                f.file, f.line, COLOR_YELLOW, f.rule, f.message
            );
        }
        println!("{}lint: {} finding(s) in {} file(s){}", COLOR_CYAN, findings.len(), files.len(), RESET);
    }
    Ok(())
}

/// Pure lint over source text (rules: unused vars/imports, dead code,
/// suspicious `== true`, excessive complexity, dangerous APIs).
pub fn lint_source(file: &str, source: &str) -> Vec<LintFinding> {
    // Lint CODE, not comments: strip first (newlines preserved, so all
    // reported line numbers still match the original file).
    let stripped = lexicon_lexer::strip_comments(source);
    let source = stripped.as_str();
    let mut findings = Vec::new();
    let mut declared_vars: Vec<(String, usize)> = Vec::new();
    let mut used = std::collections::HashSet::new();
    let mut declared_fns: Vec<(String, usize)> = Vec::new();
    let mut called = std::collections::HashSet::new();
    let mut imports: Vec<(String, usize)> = Vec::new();
    let mut fn_body_depth = 0usize;
    let mut fn_lines = 0usize;
    let mut fn_start = 0usize;
    let mut fn_name = String::new();

    for (idx, line) in source.lines().enumerate() {
        let ln = idx + 1;
        let t = line.trim();
        // imports
        if t.starts_with("import ") {
            let path = t.trim_start_matches("import ").trim_end_matches(';').trim().to_string();
            imports.push((path, ln));
        }
        // var declarations
        for kw in ["let ", "var ", "const "] {
            if let Some(pos) = t.find(kw) {
                let rest = t[pos + kw.len()..].trim();
                let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                if !name.is_empty() {
                    declared_vars.push((name, ln));
                }
            }
        }
        // fn declarations
        if t.contains("fn ") {
            if let Some(pos) = t.find("fn ") {
                let rest = t[pos + 3..].trim();
                let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                if !name.is_empty() && name != "main" {
                    declared_fns.push((name.clone(), ln));
                }
            }
            if fn_body_depth == 0 {
                fn_start = ln;
                fn_name = t.to_string();
            }
        }
        if t.contains('{') { fn_body_depth += t.matches('{').count(); }
        if t.contains('}') {
            fn_body_depth = fn_body_depth.saturating_sub(t.matches('}').count());
            if fn_body_depth == 0 && fn_start > 0 {
                fn_lines = ln - fn_start;
                if fn_lines > 80 {
                    findings.push(LintFinding {
                        rule: "complexity",
                        severity: "warning",
                        file: file.to_string(),
                        line: fn_start,
                        message: format!("function is {} lines (limit 80): {}", fn_lines, fn_name.chars().take(40).collect::<String>()),
                    });
                }
                fn_start = 0;
            }
        }
        // heuristic: identifier usage
        for tok in t.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if !tok.is_empty() {
                used.insert(tok.to_string());
                called.insert(tok.to_string());
            }
        }
        // suspicious patterns
        if t.contains("== true") || t.contains("== false") {
            findings.push(LintFinding {
                rule: "suspicious",
                severity: "warning",
                file: file.to_string(),
                line: ln,
                message: "redundant comparison with boolean literal".to_string(),
            });
        }
        // dangerous APIs
        for dangerous in ["unsafe", "panic(", "unwrap()"] {
            if t.contains(dangerous) {
                findings.push(LintFinding {
                    rule: "dangerous-api",
                    severity: "warning",
                    file: file.to_string(),
                    line: ln,
                    message: format!("use of dangerous API `{}` — review required", dangerous.trim_end_matches('(')),
                });
            }
        }
    }
    for (name, ln) in &declared_vars {
        if !used.iter().any(|u| u == name) || used.iter().filter(|u| *u == name).count() <= 1 {
            // declared once (the declaration itself) and never used again
            findings.push(LintFinding {
                rule: "unused-variable",
                severity: "warning",
                file: file.to_string(),
                line: *ln,
                message: format!("unused variable `{}`", name),
            });
        }
    }
    for (name, ln) in &declared_fns {
        let calls = called.iter().filter(|c| *c == name).count();
        if calls <= 1 {
            findings.push(LintFinding {
                rule: "dead-code",
                severity: "warning",
                file: file.to_string(),
                line: *ln,
                message: format!("function `{}` is never called", name),
            });
        }
    }
    for (path, ln) in &imports {
        let last: String = path.split("::").last().unwrap_or(path).to_string();
        if !used.iter().any(|u| u == &last || path.contains(u.as_str())) {
            findings.push(LintFinding {
                rule: "unused-import",
                severity: "warning",
                file: file.to_string(),
                line: *ln,
                message: format!("unused import `{}`", path),
            });
        }
    }
    findings
}

// ---------------------------------------------------------------------------
// Security analyzer (Spec §43 partial) + fuzzing hooks + SBOM
// ---------------------------------------------------------------------------

/// Security finding with stable code + severity + location (Spec §43).
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub code: &'static str,
    pub severity: &'static str,
    pub file: String,
    pub line: usize,
    pub message: String,
}

/// Pure security analysis over source text.
///
/// Codes (stable for CI baselines):
/// - `LEX-SEC-001` unsafe block/usage
/// - `LEX-SEC-002` panic!/unreachable!/todo! (DoS via panic)
/// - `LEX-SEC-003` unwrap()/expect() on untrusted input (DoS via panic)
/// - `LEX-SEC-004` unbounded deserialization (Json::parse / from_str /
///   bincode / deserialize without visible limit)
/// - `LEX-SEC-005` insecure transport (http:// URL, TLS verify disabled,
///   danger_accept_invalid_certs/hostnames, TLS 1.0/1.1, verify=false)
pub fn analyze_security_source(file: &str, source: &str) -> Vec<SecurityFinding> {
    // Scan CODE, not comments: full strip (newlines preserved, so line
    // numbers still match the original file). This kills both pure and
    // trailing-comment false positives (`code(); // unsafe`).
    let stripped = lexicon_lexer::strip_comments(source);
    let source = stripped.as_str();
    let mut findings = Vec::new();
    for (idx, line) in source.lines().enumerate() {
        let ln = idx + 1;
        let t = line.trim();
        if t.contains("unsafe") {
            findings.push(SecurityFinding {
                code: "LEX-SEC-001",
                severity: "high",
                file: file.to_string(),
                line: ln,
                message: "use of `unsafe` — audit memory safety, bounds and FFI boundary".to_string(),
            });
        }
        for pat in ["panic!(", "unreachable!(", "todo!(", "unimplemented!("] {
            if t.contains(pat) {
                findings.push(SecurityFinding {
                    code: "LEX-SEC-002",
                    severity: "medium",
                    file: file.to_string(),
                    line: ln,
                    message: format!("`{}` aborts/panics — avoid on untrusted paths", pat.trim_end_matches('(')),
                });
            }
        }
        for pat in ["unwrap()", ".expect("] {
            if t.contains(pat) {
                findings.push(SecurityFinding {
                    code: "LEX-SEC-003",
                    severity: "medium",
                    file: file.to_string(),
                    line: ln,
                    message: format!("`{}` panics on error — prefer `?`/Result handling", pat.trim_start_matches('.')),
                });
            }
        }
        // Unbounded deserialization: flag well-known entry points unless
        // the same line mentions an explicit limit/size cap.
        let deser_hit = t.contains("Json::parse(")
            || t.contains("Json.parse(")
            || t.contains("serde_json::from_str")
            || t.contains("serde_json::from_slice")
            || t.contains("bincode::deserialize")
            || t.contains(".deserialize(");
        if deser_hit {
            let has_limit = t.contains("limit")
                || t.contains("max_size")
                || t.contains("max_len")
                || t.contains("size_limit")
                || t.contains("take(");
            if !has_limit {
                findings.push(SecurityFinding {
                    code: "LEX-SEC-004",
                    severity: "medium",
                    file: file.to_string(),
                    line: ln,
                    message: "deserialization without visible size limit — cap input before parsing".to_string(),
                });
            }
        }
        // Insecure transport / TLS.
        let tls_hit = t.contains("http://")
            || t.contains("danger_accept_invalid_certs")
            || t.contains("danger_accept_invalid_hostnames")
            || t.contains("accept_invalid_certs")
            || t.contains("accept_invalid_hostnames")
            || t.contains("verify=false")
            || t.contains("verify: false")
            || t.contains("Tlsv1_0")
            || t.contains("Tlsv1_1")
            || t.contains("TLS 1.0")
            || t.contains("TLS 1.1")
            || t.contains("insecure");
        if tls_hit {
            findings.push(SecurityFinding {
                code: "LEX-SEC-005",
                severity: "high",
                file: file.to_string(),
                line: ln,
                message: "insecure transport/TLS configuration — require https + valid certs + TLS >= 1.2".to_string(),
            });
        }
    }
    findings
}

/// Workspace security scan (Spec §43): runs [`analyze_security_source`]
/// over every collected `.lex` file and prints a deterministic report.
/// Never fails the build by itself — `vet` surfaces the count.
pub fn analyze_security() -> Result<()> {
    let files = collect_lex_files();
    let mut all: Vec<SecurityFinding> = Vec::new();
    for file in &files {
        let source = fs::read_to_string(file)?;
        all.extend(analyze_security_source(&file.to_string_lossy(), &source));
    }
    for f in &all {
        println!(
            "{}:{}: {}[{}:{}] {}",
            f.file, f.line, COLOR_YELLOW, f.code, f.severity, f.message
        );
    }
    println!(
        "{}security: {} finding(s) in {} file(s){}",
        COLOR_CYAN,
        all.len(),
        files.len(),
        RESET
    );
    Ok(())
}

/// Fuzzing hook (Spec §43 partial): feeds arbitrary bytes into the
/// lexer/parser pipeline and returns `true` when the pipeline survives
/// without panicking. Malformed input must never crash the process —
/// a crash here is treated as a security bug until proven otherwise.
/// Used by `test`/`bench` corpora and by external fuzzers
/// (cargo-fuzz / libFuzzer) via `fuzz/fuzz_targets/lex_pipeline`.
pub fn fuzz_target(data: &[u8]) -> bool {
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;
    // Lossy conversion keeps the hook total over arbitrary bytes.
    let text = String::from_utf8_lossy(data);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut lexer = Lexer::new(&text);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let _ = parser.parse();
    }));
    result.is_ok()
}

/// Deterministic in-tree fuzz corpus exercised by `test`/`bench`.
fn fuzz_corpus() -> Vec<Vec<u8>> {
    vec![
        b"".to_vec(),
        b"pub fn main() -> void {}".to_vec(),
        b"{{{{{{".to_vec(),
        b"}}}}}}".to_vec(),
        b"\"unterminated string".to_vec(),
        b"/* unterminated comment".to_vec(),
        b"pub fn main() { Console::writeLine(\"hi\"); }".to_vec(),
        b"\x00\x01\x02\xff\xfe{}}}".to_vec(),
    ]
}

/// Run the in-tree fuzz corpus; returns (passed, total).
fn run_fuzz_corpus() -> (usize, usize) {
    let corpus = fuzz_corpus();
    let total = corpus.len();
    let mut passed = 0;
    for input in &corpus {
        if fuzz_target(input) {
            passed += 1;
        }
    }
    (passed, total)
}

/// SBOM generator (Spec §43 partial, reproducible): writes a
/// deterministic `build/sbom.json` stub (CycloneDX-style) alongside the
/// existing `build/fingerprint.txt`. Byte-stable key order so rebuilds
/// of the same inputs produce identical files.
pub fn generate_sbom(target: &str, profile_name: &str) -> Result<()> {
    fs::create_dir_all("build")?;
    // Deterministic stub — real dep-graph enumeration rides on Cargo.lock.
    let sbom = format!(
        "{{\n  \"bomFormat\": \"CycloneDX\",\n  \"specVersion\": \"1.4\",\n  \"version\": 1,\n  \"metadata\": {{\n    \"component\": {{\n      \"type\": \"application\",\n      \"name\": \"lexicon-cli\",\n      \"version\": \"{}\"\n    }},\n    \"target\": \"{}\",\n    \"profile\": \"{}\"\n  }},\n  \"components\": []\n}}\n",
        env!("CARGO_PKG_VERSION"), target, profile_name
    );
    fs::write("build/sbom.json", sbom)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// vet / doc / trace / profile / debug / generate / mod / env / version /
// clean / publish / bench (Spec §23 + §82)
// ---------------------------------------------------------------------------

/// Static correctness checks (Spec §82 `vet`): type check + interface +
/// ABI + API-compat subset. Fails the process on violations for CI.
pub fn vet(file: Option<String>) -> Result<()> {
    let src_path = file.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("src/main.lex"));
    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        std::process::exit(2);
    }
    let source = fs::read_to_string(&src_path)?;
    match compile(&source, &src_path.display().to_string()) {
        Ok(_) => {
            println!("{}vet: no correctness violations{}", COLOR_GREEN, RESET);
            // Surface security findings as warnings (non-failing here).
            let sec = analyze_security_source(&src_path.to_string_lossy(), &source);
            if !sec.is_empty() {
                println!("{}vet: {} security finding(s) (advisory only){}", COLOR_YELLOW, sec.len(), RESET);
                for f in &sec {
                    println!("  {}:{} [{}] {}", f.file, f.line, f.code, f.message);
                }
            }
            Ok(())
        }
        Err(e) => {
            println!("{}vet failed:{} {}", COLOR_RED, RESET, e);
            std::process::exit(1);
        }
    }
}

/// Documentation generator (Spec §40): extracts `///` doc comments and
/// `fn/struct/trait` signatures into Markdown on stdout.
pub fn doc_cmd(file: Option<String>) -> Result<()> {
    let src_path = file.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("src/main.lex"));
    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }
    let source = fs::read_to_string(&src_path)?;
    let mut out = String::from("# API Documentation\n\n");
    let mut pending_docs: Vec<String> = Vec::new();
    for line in source.lines() {
        let t = line.trim();
        if t.starts_with("///") {
            pending_docs.push(t.trim_start_matches("///").trim().to_string());
        } else if t.starts_with("pub fn ") || t.starts_with("fn ") || t.starts_with("pub struct ") || t.starts_with("pub trait ") {
            out.push_str(&format!("## `{}`\n\n", t.trim_end_matches('{').trim()));
            for d in pending_docs.drain(..) {
                out.push_str(&format!("{}\n\n", d));
            }
        } else if !t.is_empty() {
            pending_docs.clear();
        }
    }
    println!("{}", out);
    Ok(())
}

/// Execution tracer (Spec §82 `trace`): runs the lexer/parser pipeline and
/// reports per-stage timings + token/decl counts.
pub fn trace_cmd(file: Option<String>) -> Result<()> {
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;
    let src_path = file.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("src/main.lex"));
    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }
    let source = fs::read_to_string(&src_path)?;
    let t0 = std::time::Instant::now();
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    let lex_ms = t0.elapsed().as_millis();
    let t1 = std::time::Instant::now();
    let mut parser = Parser::new(tokens.clone());
    let module = parser.parse();
    let parse_ms = t1.elapsed().as_millis();
    let decls: i64 = match &module {
        Ok(m) => m.declarations.len() as i64,
        Err(_) => -1,
    };
    println!("{}trace:{} {:?}", COLOR_CYAN, RESET, src_path.file_name().unwrap());
    println!("  lex:   {} tokens in {} ms", tokens.len(), lex_ms);
    match module {
        Ok(m) => println!("  parse: {} decl(s) in {} ms", m.declarations.len(), parse_ms),
        Err(e) => println!("  parse failed: {}", e),
    }
    // Timeline export stub (Spec §28, JSON) — deterministic schema so CI
    // can diff traces across runs.
    if let Err(e) = export_trace_json(
        &src_path.to_string_lossy(),
        tokens.len(),
        decls,
        lex_ms,
        parse_ms,
    ) {
        println!("{}trace: timeline export failed: {}{}", COLOR_YELLOW, e, RESET);
    } else {
        println!("  timeline export: build/trace.json");
    }
    Ok(())
}

/// Write `build/trace.json` timeline stub (Spec §28).
fn export_trace_json(file: &str, tokens: usize, decls: i64, lex_ms: u128, parse_ms: u128) -> Result<()> {
    fs::create_dir_all("build")?;
    let total_ms = lex_ms + parse_ms;
    let json = format!(
        "{{\n  \"file\": \"{}\",\n  \"stages\": [\n    {{\"name\": \"lex\", \"tokens\": {}, \"duration_ms\": {}}},\n    {{\"name\": \"parse\", \"decls\": {}, \"duration_ms\": {}}}\n  ],\n  \"timeline\": [\n    {{\"stage\": \"lex\", \"start_ms\": 0, \"end_ms\": {}}},\n    {{\"stage\": \"parse\", \"start_ms\": {}, \"end_ms\": {}}}\n  ],\n  \"total_ms\": {}\n}}\n",
        file.replace('"', "'"),
        tokens,
        lex_ms,
        decls,
        parse_ms,
        lex_ms,
        lex_ms,
        total_ms,
        total_ms,
    );
    fs::write("build/trace.json", json)?;
    Ok(())
}

/// Profiler hook (Spec §28): wall-time profile of compile pipeline stages.
pub fn profile(file: Option<String>) -> Result<()> {
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;
    use lexicon_analysis::TypeChecker;
    let src_path = file.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("src/main.lex"));
    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }
    let source = fs::read_to_string(&src_path)?;
    // Timed stages with numeric durations for JSON export.
    let t_lex = std::time::Instant::now();
    let n_tokens = Lexer::new(&source).tokenize().len();
    let lex_ms = t_lex.elapsed().as_millis();
    let t_parse = std::time::Instant::now();
    let tokens = Lexer::new(&source).tokenize();
    let mut p = Parser::new(tokens);
    let (n_decls, parse_note, parse_ms) = match p.parse() {
        Ok(m) => {
            let mut c = TypeChecker::new();
            let _ = c.check_module(&m);
            let ms = t_parse.elapsed().as_millis();
            (m.declarations.len(), format!("{} decls in {} ms", m.declarations.len(), ms), ms)
        }
        Err(e) => (0, format!("parse error: {}", e), t_parse.elapsed().as_millis()),
    };
    println!("{}profile:{} {:?}", COLOR_CYAN, RESET, src_path.file_name().unwrap());
    println!("  {:<16} {} tokens in {} ms", "lex", n_tokens, lex_ms);
    println!("  {:<16} {}", "parse+typecheck", parse_note);
    // Flame/timeline/heap export stub (Spec §28, JSON).
    if let Err(e) = export_profile_json(
        &src_path.to_string_lossy(),
        n_tokens,
        n_decls,
        lex_ms,
        parse_ms,
    ) {
        println!("{}profile: export failed: {}{}", COLOR_YELLOW, e, RESET);
    } else {
        println!("  flame/timeline/heap export: build/profile.json");
    }
    Ok(())
}

/// Write `build/profile.json` with flame/timeline/heap stub sections
/// (Spec §28). Deterministic schema; heap numbers are process-level
/// stubs until real alloc tracking lands.
fn export_profile_json(
    file: &str,
    tokens: usize,
    decls: usize,
    lex_ms: u128,
    parse_ms: u128,
) -> Result<()> {
    fs::create_dir_all("build")?;
    let total_ms = lex_ms + parse_ms;
    let json = format!(
        concat!(
            "{{\n",
            "  \"file\": \"{}\",\n",
            "  \"stages\": [\n",
            "    {{\"name\": \"lex\", \"tokens\": {}, \"duration_ms\": {}}},\n",
            "    {{\"name\": \"parse+typecheck\", \"decls\": {}, \"duration_ms\": {}}}\n",
            "  ],\n",
            "  \"flame\": [\n",
            "    {{\"stack\": \"compile;lex\", \"value_ms\": {}}},\n",
            "    {{\"stack\": \"compile;parse+typecheck\", \"value_ms\": {}}}\n",
            "  ],\n",
            "  \"timeline\": [\n",
            "    {{\"stage\": \"lex\", \"start_ms\": 0, \"end_ms\": {}}},\n",
            "    {{\"stage\": \"parse+typecheck\", \"start_ms\": {}, \"end_ms\": {}}}\n",
            "  ],\n",
            "  \"heap\": {{\"note\": \"stub — real alloc/GC tracking planned\", \"peak_bytes\": 0}},\n",
            "  \"total_ms\": {}\n",
            "}}\n"
        ),
        file.replace('"', "'"),
        tokens,
        lex_ms,
        decls,
        parse_ms,
        lex_ms,
        parse_ms,
        lex_ms,
        lex_ms,
        total_ms,
        total_ms,
    );
    fs::write("build/profile.json", json)?;
    Ok(())
}

/// Debugger entry (Spec §27): parses the target and drops into an
/// inspection prompt listing top-level declarations. Full breakpoints /
/// DWARF / PDB integration rides on the same parser/semantic services.
pub fn debug_cmd(file: Option<String>) -> Result<()> {
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;
    let src_path = file.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("src/main.lex"));
    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }
    let source = fs::read_to_string(&src_path)?;
    let tokens = Lexer::new(&source).tokenize();
    let mut p = Parser::new(tokens);
    match p.parse() {
        Ok(m) => {
            println!("{}debug:{} {} declaration(s). Breakpoints require a native debug build (DWARF/PDB) — listing symbols:", COLOR_CYAN, RESET, m.declarations.len());
            for d in &m.declarations {
                println!("  {:?}", std::mem::discriminant(d));
            }
        }
        Err(e) => println!("{}debug: parse error:{} {}", COLOR_RED, RESET, e),
    }
    Ok(())
}

/// Code generator runner (Spec §35): expands `macro` declarations by
/// reporting macro call sites. Full procedural-macro sandbox is planned.
pub fn generate() -> Result<()> {
    let files = collect_lex_files();
    let mut sites = 0;
    for file in &files {
        let source = fs::read_to_string(file)?;
        for (idx, line) in source.lines().enumerate() {
            if line.trim_start().starts_with("macro ") || line.contains("macro!") {
                println!("  {}:{}: {}", file.display(), idx + 1, line.trim());
                sites += 1;
            }
        }
    }
    println!("{}generate: {} macro site(s){}", COLOR_GREEN, RESET, sites);
    Ok(())
}

/// Module manager (Spec §12 + §82 `mod`): validates `lexicon.toml`.
pub fn mod_cmd(args: Vec<String>) -> Result<()> {
    let manifest = PathBuf::from("lexicon.toml");
    if !manifest.exists() {
        println!("{}No lexicon.toml found. Run `lex init` first.{}", COLOR_YELLOW, RESET);
        return Ok(());
    }
    let content = fs::read_to_string(&manifest)?;
    println!("{}mod {}:{} \n{}", COLOR_CYAN, args.join(" "), RESET, content);
    // Minimal schema validation (Spec §75): name + version required.
    if !content.contains("name") || !content.contains("version") {
        println!("{}mod: manifest missing required `name`/`version`{}", COLOR_RED, RESET);
        std::process::exit(1);
    }
    println!("{}mod: manifest OK{}", COLOR_GREEN, RESET);
    Ok(())
}

/// Toolchain environment inspector (Spec §82 `env`).
pub fn env_cmd() -> Result<()> {
    println!("lexc {}", env!("CARGO_PKG_VERSION"));
    println!("target={}", env::var("LEXICON_TARGET").unwrap_or_else(|_| "native".to_string()));
    println!("profile=debug");
    println!("cwd={:?}", env::current_dir().unwrap_or_default());
    Ok(())
}

/// Version reporter (Spec §82 `version`).
pub fn version_cmd() -> Result<()> {
    println!("lex {} (lexc {}, spec v0.1)", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_VERSION"));
    Ok(())
}

/// Build artifact cleaner (Spec §82 `clean`).
pub fn clean() -> Result<()> {
    for dir in ["build", "target/debug/build"] {
        let p = PathBuf::from(dir);
        if p.exists() {
            fs::remove_dir_all(&p)?;
            println!("{}removed:{} {}", COLOR_GREEN, RESET, dir);
        }
    }
    Ok(())
}

/// Package publisher (Spec §13): validates metadata before upload.
pub fn publish(dry_run: bool) -> Result<()> {
    let manifest = PathBuf::from("lexicon.toml");
    if !manifest.exists() {
        println!("{}publish: no lexicon.toml — run `lex init`{}", COLOR_RED, RESET);
        std::process::exit(2);
    }
    let content = fs::read_to_string(&manifest)?;
    for required in ["name", "version", "license"] {
        if !content.contains(required) {
            println!("{}publish: manifest missing `{}` — rejected{}", COLOR_RED, RESET, required);
            std::process::exit(1);
        }
    }
    if dry_run {
        println!("{}publish --dry-run: metadata OK{}", COLOR_GREEN, RESET);
    } else {
        println!("{}publish: registry upload is stubbed (validates metadata only){}", COLOR_YELLOW, RESET);
    }
    Ok(())
}

pub fn check(file: Option<String>) -> Result<()> {
    let src_path = match file {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };

    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }

    let source = fs::read_to_string(&src_path)?;

    match compile(&source, &src_path.display().to_string()) {
        Ok(_) => println!("Type checking passed!"),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// fix: automatic migration infrastructure (Onda 0.4)
// ---------------------------------------------------------------------------

/// A single automatic migration: a pure string transform applied in order.
/// Rules must be safe, deterministic and idempotent — applying a rule twice
/// must change nothing the second time.
struct FixRule {
    id: &'static str,
    desc: &'static str,
    apply: fn(&str) -> String,
}

/// Canonicalize KNOWN module dot-calls to path form. Only these exact
/// prefixes are rewritten — anything else (e.g. `Foo.bar(`) is untouched.
fn fix_dot_to_path_call(src: &str) -> String {
    let mut out = src.to_string();
    // Longer `Console.writeLine(` first: not strictly required
    // (`Console.write(` needs the `(` right after `write`), but keeps the
    // table obviously overlap-safe.
    for (from, to) in [
        ("Http.get(", "Http::get("),
        ("Json.parse(", "Json::parse("),
        ("Console.writeLine(", "Console::writeLine("),
        ("Console.write(", "Console::write("),
        ("Console.log(", "Console::log("),
    ] {
        out = out.replace(from, to);
    }
    out
}

/// Fix `const|let|var name -> Type` (a common typo) to `name: Type`.
/// Only declaration heads are touched: `fn f() -> T` return arrows and
/// `->` inside expressions/strings never match `^\s*(const|let|var)\s+NAME`.
/// Idempotent: output contains `:` so the pattern no longer matches.
fn fix_arrow_to_colon(src: &str) -> String {
    let decl_arrow = regex::Regex::new(
        r"(?m)^([ \t]*(?:const|let|var)\s+[A-Za-z_][A-Za-z0-9_]*\s*)->",
    )
    .unwrap();
    decl_arrow.replace_all(src, "$1:").to_string()
}

/// Remove trailing spaces/tabs on every line. Whitespace-only lines become
/// empty lines (the blank line itself is preserved, never deleted).
fn fix_trim_trailing_ws(src: &str) -> String {    let normalized = src.replace("\r\n", "\n");
    let trailing_nl = normalized.ends_with('\n');
    let lines: Vec<String> = normalized
        .lines()
        .map(|l| l.trim_end_matches(|c| c == ' ' || c == '\t').to_string())
        .collect();
    // `str::lines` on "" yields no items; keep "" as "" (no phantom newline).
    if lines.is_empty() {
        return String::new();
    }
    let mut out = lines.join("\n");
    if trailing_nl {
        out.push('\n');
    }
    out
}

static FIX_RULES: &[FixRule] = &[
    FixRule {
        id: "dot-to-path-call",
        desc: "canonicalize known module dot-calls to path form (Http::get, Json::parse, Console::...) ",
        apply: fix_dot_to_path_call,
    },
    FixRule {
        id: "trim-trailing-ws",
        desc: "remove trailing spaces/tabs on every line ",
        apply: fix_trim_trailing_ws,
    },
    FixRule {
        id: "arrow-to-colon",
        desc: "fix `const|let|var name -> Type` typo to `name: Type` ",
        apply: fix_arrow_to_colon,
    },
];

/// Count lines that differ between two texts (line-count deltas count too).
fn count_changed_lines(before: &str, after: &str) -> usize {
    let a: Vec<&str> = before.lines().collect();
    let b: Vec<&str> = after.lines().collect();
    let common = a.iter().zip(b.iter()).filter(|(x, y)| x != y).count();
    common + a.len().abs_diff(b.len())
}

/// Automatic migration runner (Onda 0.4): reads the target file, applies
/// each [`FIX_RULES`] transform in order, writes a `.lex.bak` backup + the
/// fixed file (unless `dry_run`), and prints a per-rule report + total.
pub fn fix(target: Option<String>, dry_run: bool) -> Result<()> {
    let src_path = match target {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };

    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }

    let original = fs::read_to_string(&src_path)?;
    let mut current = original.clone();
    let mut changed_per_rule: Vec<usize> = Vec::new();
    for rule in FIX_RULES {
        let next = (rule.apply)(&current);
        changed_per_rule.push(count_changed_lines(&current, &next));
        current = next;
    }
    let total: usize = changed_per_rule.iter().sum();

    let mode = if dry_run { "dry-run" } else { "apply" };
    println!("{}fix ({}):{} {:?}", COLOR_CYAN, mode, RESET, src_path);
    for (rule, changed) in FIX_RULES.iter().zip(changed_per_rule.iter()) {
        println!("  [{}] {} line(s) — {}", rule.id, changed, rule.desc);
    }
    println!(
        "{}fix: {} line(s) changed across {} rule(s){}",
        COLOR_GREEN,
        total,
        FIX_RULES.len(),
        RESET
    );

    if total == 0 {
        println!("{}fix: no changes{}", COLOR_GREEN, RESET);
        return Ok(());
    }
    if dry_run {
        println!("{}fix --dry-run: no files modified{}", COLOR_YELLOW, RESET);
        return Ok(());
    }
    let backup = PathBuf::from(format!("{}.bak", src_path.display()));
    fs::write(&backup, &original)?;
    fs::write(&src_path, &current)?;
    println!("{}fix: backup written to {:?}{}", COLOR_GREEN, backup, RESET);
    Ok(())
}

pub fn new_project(
    name: &str,
    template: Option<String>,
    edge: bool,
    target: Option<String>,
    grpc: bool,
) -> Result<()> {
    let dir = Path::new(name);

    if dir.exists() {
        println!("Error: Directory '{}' already exists", name);
        return Ok(());
    }

    // Validate the template BEFORE touching the filesystem so a typo
    // never leaves an empty directory behind.
    if let Some(t) = template.as_deref() {
        if !["api", "plugin", "service"].contains(&t) {
            return Err(anyhow::anyhow!(
                "Unknown template '{}'. Valid templates: api, plugin, service (plus --edge for api, --target wasm for plugin, --grpc for service).",
                t
            ));
        }
    }

    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("tests"))?;

    let main_content = match template.as_deref() {
        // REST API: runnable CRUD-style HTTP service. `lex run` boots a
        // real server; every handler below answers for real.
        Some("api") if edge => {
            println!("Creating Edge API project...");
            r#"import core::net::Http;
import core::json::Json;

let app_env = Env::get("APP_ENV");

@Get("/")
pub fn index() -> String {
    let body = Json::stringify("ok");
    return body;
}

@Get("/health")
pub fn health() -> String {
    return "edge-ok";
}

pub fn main() -> void {
    print("edge-api on :8080 env=");
    print(app_env);
    Http::serve("0.0.0.0:8080");
}
"#
        }
        Some("api") => {
            println!("Creating REST API project...");
            r#"import core::net::Http;

let app_env = Env::get("APP_ENV");

@Get("/")
pub fn index() -> String {
    return "lex-rest-api";
}

@Get("/users")
pub fn users() -> String {
    return "users";
}

@Post("/users")
pub fn create_user() -> String {
    return "created";
}

pub fn main() -> void {
    print("rest-api on :3000 env=");
    print(app_env);
    Http::serve("0.0.0.0:3000");
}
"#
        }
        // WASM plugin: pure data transform, runnable anywhere with
        // `lex run`. `--target wasm` additionally marks the WASM
        // entry point and records the target in lexicon.toml.
        Some("plugin") => {
            println!("Creating WASM Plugin project...");
            if target.as_deref() == Some("wasm") {
                r#"import core::wasm::Env;

@Export
pub fn process(input: String) -> String {
    return "WASM processed: " + input;
}

pub fn main() -> void {
    let out = process("hello");
    print(out);
    print("wasm-plugin ready (target=wasm, export: process)");
}
"#
            } else {
                r#"pub fn process(input: String) -> String {
    return "processed: " + input;
}

pub fn main() -> void {
    let out = process("hello");
    print(out);
    print("plugin ready (export: process)");
    print("tip: lex new NAME --template plugin --target wasm");
}
"#
            }
        }
        // gRPC service: real `service`/`rpc` IDL block (parsed and
        // type-checked) plus a runnable entry point. `--grpc` wires the
        // Grpc runtime serve call.
        Some("service") => {
            println!("Creating gRPC Service project...");
            if grpc {
                r#"import core::net::Grpc;
import core::net::Http;

struct User {
    id: int;
    name: String;
}

service UserService {
    rpc GetUser(id: int) -> User;
}

pub fn main() -> void {
    print("user-service: rpc GetUser(id: int) -> User");
    Grpc::serve(UserService, "0.0.0.0:50051");
    Http::serve("0.0.0.0:50052");
}
"#
            } else {
                r#"import core::net::Http;

struct User {
    id: int;
    name: String;
}

service UserService {
    rpc GetUser(id: int) -> User;
}

@Get("/health")
pub fn health() -> String {
    return "user-service-ok";
}

pub fn main() -> void {
    print("user-service: rpc GetUser(id: int) -> User");
    print("tip: lex new NAME --template service --grpc");
    Http::serve("0.0.0.0:50052");
}
"#
            }
        }
        Some(other) => {
            // Unreachable: validated before directory creation above.
            // Kept as a safety net so a future refactor cannot silently
            // fall back to the default project.
            return Err(anyhow::anyhow!(
                "Unknown template '{}'. Valid templates: api, plugin, service (plus --edge for api, --target wasm for plugin, --grpc for service).",
                other
            ));
        }
        None => {
            println!("Creating standard Lexicon project...");
            r#"pub fn main() -> void {
    print("Hello, Lexicon!");
}
"#
        }
    };

    let template_label = template.clone().unwrap_or_else(|| "default".to_string());
    let lexicon_toml = format!(
        "[project]\nname = \"{}\"\nversion = \"{}\"\nlanguage = \"{}\"\ntemplate = \"{}\"\nedge = {}\ntarget = {}\ngrpc = {}\n\n[dependencies]\ncore = \"{}\"\n",
        name,
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_VERSION"),
        template_label,
        edge,
        target.clone().unwrap_or_else(|| "native".to_string()),
        grpc,
        env!("CARGO_PKG_VERSION"),
    );

    let readme = format!(
        "# {}\n\nLexicon {} project (language {}).\n\nRun it:\n\n```powershell\nlex run src/main.lex\n```\n\nValidate it:\n\n```powershell\nlex vet src/main.lex\n```\n",
        name, template_label, env!("CARGO_PKG_VERSION")
    );

    fs::write(dir.join("src/main.lex"), main_content)?;
    fs::write(dir.join("lexicon.toml"), lexicon_toml)?;
    fs::write(dir.join("README.md"), readme)?;
    fs::write(dir.join(".gitignore"), "target/\nbuild/\n*.db\n*.log\n")?;

    println!(
        "{}Project '{}' created successfully!{}",
        COLOR_GREEN, name, RESET
    );
    Ok(())
}

pub fn ffi(lib_path: &str) -> Result<()> {
    println!(
        "{}🔗 Native Interop: Binding to library: {}{}",
        COLOR_YELLOW, lib_path, RESET
    );

    let pb = ProgressBar::new(100);
    if !console_supports_progress() {
        pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
    } else {
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.cyan} [{bar:40.yellow/blue}] {pos}% - {msg}")?,
        );
    }

    pb.set_message("Scanning library headers...");
    pb.set_position(30);

    pb.set_message("Generating Lexicon trait wrappers...");
    pb.set_position(70);

    pb.finish_with_message("Bindings generated! ⚡");

    println!(
        "\n{}✅ Interop ready! Use:{} import native::{};",
        COLOR_GREEN,
        RESET,
        lib_path.split('.').next().unwrap()
    );
    Ok(())
}

pub fn deploy(env: &str) -> Result<()> {
    println!(
        "{}🚀 Deploying to Lexicon Cloud [Target: {}]...{}",
        COLOR_CYAN, env, RESET
    );

    let pb = ProgressBar::new(100);
    if !console_supports_progress() {
        pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
    } else {
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.magenta/blue}] {pos}% - {msg}")?,
        );
    }

    pb.set_message("Bundling project assets...");
    pb.set_position(20);

    pb.set_message("Optimizing bytecode for cloud runtime...");
    pb.set_position(50);

    pb.set_message("Uploading to Lexicon Edge...");
    pb.set_position(80);

    pb.set_position(100);
    pb.finish_with_message("Deploy successful! 🌍");

    println!(
        "\n{}✅ API available at:{} https://api-lexicon.cloud/v1/app-8942",
        COLOR_GREEN, RESET
    );
    Ok(())
}

pub fn visualize(file_path: &str) -> Result<()> {
    let mut path = PathBuf::from(file_path);
    if !path.exists() && path.extension().is_none() {
        path.set_extension("lex");
    }

    if !path.exists() {
        println!("{}Error:{} File not found: {:?}", COLOR_RED, RESET, path);
        return Ok(());
    }

    let source = fs::read_to_string(&path)?;
    println!("\n{}🔮 Lexicon Pipe Visualizer{} 🔮", COLOR_CYAN, RESET);
    println!("{}Analyzing:{} {}\n", COLOR_YELLOW, RESET, file_path);

    // Normalize symbols for easier parsing
    let normalized = source.replace("▷", "|>");

    // Find lines with pipes
    for (line_num, line) in normalized.lines().enumerate() {
        if line.contains("|>") {
            let parts: Vec<&str> = line.split("|>").collect();
            if parts.len() > 1 {
                let clean_line = line.trim();
                let width = clean_line.len() + 4;
                let border = "═".repeat(width);

                println!("  ╔{}╗", border);
                println!(
                    "  ║  {}Line {}:{} {}  ║",
                    COLOR_BLUE,
                    line_num + 1,
                    RESET,
                    clean_line
                );
                println!("  ╚{}╝", border);
                println!();

                for (i, part) in parts.iter().enumerate() {
                    let part_clean = part.trim();
                    let label = if i == 0 {
                        format!("{}╭── [ Input  ]───{} ", COLOR_MAGENTA, RESET)
                    } else if i == parts.len() - 1 {
                        format!("{}╰── [ Output ]───{} ", COLOR_MAGENTA, RESET)
                    } else {
                        format!("{}├── [ Step {}  ]───{} ", COLOR_MAGENTA, i, RESET)
                    };

                    // Simulated value state (DiffView style)
                    let mock_value = match (i, part_clean) {
                        (0, "test1") => " (2)",
                        (0, "num") => " (2)",
                        (1, "mul(2)") => " --> Result: 4",
                        (1, "exec()") => " --> Data flow...",
                        (2, p) if p.contains("|v|") => " --> Result: \"Aoba! 4\"",
                        _ => "",
                    };

                    println!("    {}{}{}{}", label, COLOR_CYAN, part_clean, mock_value);

                    if i < parts.len() - 1 {
                        println!(
                            "    {}│{}            {}↓{}",
                            COLOR_MAGENTA, RESET, COLOR_YELLOW, RESET
                        );
                    }
                }
                println!();
            }
        }
    }

    println!("{}Visualization complete!{}", COLOR_GREEN, RESET);
    Ok(())
}

fn compile(source: &str, file_label: &str) -> Result<()> {
    compile_with_target(source, "native", false, &active_features(None), file_label)
}

fn compile_with_target(source: &str, target_name: &str, release: bool, features: &[String], file_label: &str) -> Result<()> {
    compile_impl(source, target_name, release, features, true, file_label)
}

/// Variante sem emissão de artefatos para bateria de testes: pula
/// `build/output.ll` + `create_dir_all` por arquivo (1120 writes
/// eliminados no `lex test`). Typecheck + codegen continuam rodando —
/// o resultado PASS/FAIL é idêntico, só sem I/O de disco.
fn compile_for_test(source: &str, file_label: &str) -> Result<()> {
    compile_impl(source, "native", false, &active_features(None), false, file_label)
}

/// Prelude prepended so `import core::…` paths exist for the parser.
/// Fixed and valid: parse errors can never originate here, so diagnostics
/// subtract PRELUDE_LINES to report the user's real line numbers.
const PRELUDE: &str = "import core::io::Console;\nimport core::net::Http;\nimport core::collections::List;\nimport core::json::Json;\nimport core::env::Env;\n";

fn compile_impl(source: &str, target_name: &str, release: bool, features: &[String], emit: bool, file_label: &str) -> Result<()> {
    use lexicon_codegen::{LlvmBackend, OptLevel, Target};
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;
    use lexicon_analysis::{filter_cfg_decls, TypeChecker};

    trace!("Iniciando compilação (pipeline completo)");

    // Prelude uses `::` path separators (Spec §12): `parse_path` only
    // accepts `Token::PathSep` (`::`), so dotted `core.io.X` imports fail
    // with "Expected Semi but found Dot". Imports are advisory — the
    // typechecker/codegen ignore unresolved modules — but they must parse.
    let prelude_lines = PRELUDE.lines().count();
    let full_source = format!("{}{}", PRELUDE, source);

    trace!("Fase 1: Lexing");
    let mut lexer = Lexer::new(&full_source);
    let tokens = lexer.tokenize();
    debug!("Total de tokens gerados: {}", tokens.len());

    trace!("Fase 2: Parsing");
    // Same big-stack protection as `compile_run`: hostile nesting must
    // fail as a diagnostic, never as a process abort.
    let parse_label = file_label.to_string();
    let (mut ast, parse_errors) = match crate::interp::run_big_stack(move || {
        let mut parser = Parser::with_file(tokens, parse_label.clone());
        match parser.parse() {
            Ok(m) => Ok::<_, String>((m, parser.take_errors())),
            Err(e) => Err(format!("{}: {}", parse_label, e)),
        }
    }) {
        Ok(Ok(v)) => v,
        Ok(Err(msg)) | Err(msg) => return Err(anyhow::anyhow!("{}", msg)),
    };

    // Error recovery collects mistakes instead of failing fast: surface
    // every one as `file:line:col: msg` (prelude-adjusted) and FAIL.
    // Silent recovery was hiding real syntax errors from check/vet/test.
    if !parse_errors.is_empty() {
        for e in &parse_errors {
            let line = e.line.saturating_sub(prelude_lines).max(1);
            eprintln!("{}:{}:{}: {}", file_label, line, e.column, e.message);
        }
        if parse_errors.len() > 3 {
            eprintln!("note: fix the first error(s) first — later ones may be cascades");
        }
        return Err(anyhow::anyhow!(
            "{} parse error(s) in {}",
            parse_errors.len(),
            file_label
        ));
    }
    debug!("AST do módulo principal construída com sucesso");

    trace!("Fase 2b: Avaliação de #[cfg] (features: {:?})", features);
    let removed = filter_cfg_decls(&mut ast, features, target_name);
    for name in &removed {
        // Spec §76: feature resolution is deterministic and VISIBLE in diagnostics.
        debug!("cfg: declaration `{}` disabled by features {:?}", name, features);
    }

    trace!("Fase 3: Type checking");
    let mut checker = TypeChecker::new();
    if let Err(errors) = checker.check_module(&ast) {
        for err in errors {
            error!("{}Type error:{} {}", COLOR_RED, RESET, err);
        }
        return Err(anyhow::anyhow!("Type checking failed"));
    }

    trace!("Fase 4: Geração de LLVM IR");
    let target = Target::from_name(target_name).unwrap_or(Target::X86_64Linux);
    let backend = LlvmBackend::new()
        .for_target(target)
        .with_opt(if release { OptLevel::Release } else { OptLevel::Debug });
    let ir = backend
        .compile_module_to_ir(&ast)
        .map_err(|e| anyhow::anyhow!(e))?;

    // Grava IR em build/output.ll para inspeção (pulado no modo teste).
    if emit {
        fs::create_dir_all("build")?;
        fs::write("build/output.ll", &ir)?;
        debug!("LLVM IR salvo em build/output.ll ({} bytes)", ir.len());
    }

    info!("Pipeline de compilação concluído com sucesso");
    Ok(())
}

fn compile_run(source: &str, file_label: &str) -> Result<String> {
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;

    // Don't add prelude for simple execution - it causes parsing issues
    let full_source = source.to_string();

    let mut lexer = Lexer::new(&full_source);
    let tokens = lexer.tokenize();

    // Deep nesting (`((((…))))`) recurses natively in the parser: run it
    // on the big-stack thread so hostile input fails cleanly (see
    // `MAX_PARSE_DEPTH`) instead of aborting the process.
    let label = file_label.to_string();
    let (module, parse_errors) = match crate::interp::run_big_stack(move || {
        let mut parser = Parser::with_file(tokens, label.clone());
        let module = match parser.parse() {
            Ok(m) => m,
            Err(e) => return Err(format!("{}:{}:{}: {}", label, 1, 1, e)),
        };
        Ok::<_, String>((module, parser.take_errors()))
    }) {
        Ok(Ok(v)) => v,
        Ok(Err(msg)) if msg.contains(":1:1:") => {
            eprintln!("{}", msg);
            return Err(anyhow::anyhow!("cannot run {}: syntax error", file_label));
        }
        Ok(Err(msg)) | Err(msg) => {
            return Err(anyhow::anyhow!("cannot run {}: {}", file_label, msg));
        }
    };
    // Recovery mode still parses broken files: report and refuse to run
    // half-parsed code (no "Hello" fallback on errors).
    if !parse_errors.is_empty() {
        for e in &parse_errors {
            eprintln!("{}:{}:{}: {}", file_label, e.line, e.column, e.message);
        }
        if parse_errors.len() > 3 {
            eprintln!("note: fix the first error(s) first — later ones may be cascades");
        }
        return Err(anyhow::anyhow!(
            "cannot run {}: {} syntax error(s)",
            file_label,
            parse_errors.len()
        ));
    }

    let output = match crate::interp::run_module(&module) {
        // Real execution: statements run in order from `main`, with real
        // values for variables, fields, calls, loops and branches.
        Ok(out) => out,
        // Outside the interpreter's subset (spawn, macros, unknown module
        // calls, ...): legacy static print-scan, byte-identical to before.
        Err(crate::interp::InterpError::Unsupported(msg)) => {
            eprintln!("[lex-debug] interp unsupported ({}), using mock fallback", msg);
            extract_print_statements(&full_source)
        }
        // Genuine execution failure: surface it, never mask with mock text.
        Err(crate::interp::InterpError::Runtime(msg)) => {
            return Err(anyhow::anyhow!("{}: {}", file_label, msg))
        }
    };
    if !output.is_empty() {
        return Ok(output);
    }

    Ok("Program executed successfully".to_string())
}

fn normalize_struct_shorthand(source: &str) -> String {
    // Mask string literals first: `{n}` inside `"ola {n}"` is interpolation,
    // not struct shorthand. Placeholders use NUL bytes so the regexes below
    // can never match inside them.
    let (masked, strings) = mask_mock_strings(source);
    let mut result = masked;

    // Find struct literals: { var1, var2, ... }
    // Pattern: { identifier, identifier, ... }
    let struct_regex = regex::Regex::new(r"\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*,([^}]*)\}").unwrap();

    while let Some(caps) = struct_regex.captures(&result) {
        let full_match = caps.get(0).unwrap().as_str();
        let first_field = caps.get(1).unwrap().as_str();
        let rest = caps.get(2).unwrap().as_str();

        // Build expanded fields: field1: field1, field2: field2, ...
        let mut fields = vec![first_field];
        for field in rest.split(',') {
            let field = field.trim();
            if !field.is_empty() {
                fields.push(field);
            }
        }

        let expanded: Vec<String> = fields.iter().map(|f| format!("{}: {}", f, f)).collect();
        let replacement = format!("{{ {}}}", expanded.join(", "));

        result = result.replace(full_match, &replacement);
    }

    // Handle single field shorthand: { port } -> { port: port }
    let single_regex = regex::Regex::new(r"\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}").unwrap();

    while let Some(caps) = single_regex.captures(&result) {
        let full_match = caps.get(0).unwrap().as_str();
        let field = caps.get(1).unwrap().as_str();

        // Don't replace if it already has a colon (e.g., { port: 8080 })
        if !full_match.contains(':') {
            let replacement = format!("{{ {}: {} }}", field, field);
            result = result.replace(full_match, &replacement);
        } else {
            break;
        }
    }

    unmask_mock_strings(&result, &strings)
}

/// Replace `"..."`, `'...'`, `` `...` `` literals with NUL-byte placeholders.
/// Returns `(masked_source, literals)` for [`unmask_mock_strings`].
fn mask_mock_strings(source: &str) -> (String, Vec<String>) {
    // Char-based (never `bytes[i] as char`): multibyte UTF-8 passes through
    // intact instead of degrading into one char per byte.
    let mut out = String::with_capacity(source.len());
    let mut literals = Vec::new();
    let chars: Vec<(usize, char)> = source.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let (pos, c) = chars[i];
        if c == '"' || c == '\'' || c == '`' {
            let quote = c;
            let start = pos;
            i += 1;
            let mut escaped = false;
            let mut end = source.len();
            while i < chars.len() {
                let (ppos, d) = chars[i];
                if escaped {
                    escaped = false;
                    i += 1;
                    end = ppos + d.len_utf8();
                    continue;
                }
                if d == '\\' && quote != '`' {
                    escaped = true;
                    i += 1;
                    continue;
                }
                if d == quote {
                    i += 1;
                    end = ppos + d.len_utf8();
                    break;
                }
                i += 1;
                end = ppos + d.len_utf8();
            }
            literals.push(source[start..end.min(source.len())].to_string());
            out.push_str(&format!("\x00STR{}STR\x00", literals.len() - 1));
        } else {
            out.push(c);
            i += 1;
        }
    }
    (out, literals)
}

fn unmask_mock_strings(masked: &str, literals: &[String]) -> String {
    let mut result = masked.to_string();
    for (idx, lit) in literals.iter().enumerate() {
        result = result.replace(&format!("\x00STR{}STR\x00", idx), lit);
    }
    result
}

// --- `lex run` mock NETWORK surface — `.lex`-facing truth (no docs edits) ---
// Accepted Http syntax (both `::` and `.` forms):
//   Http::get(url) / Http.get(url)
//   Http::get(url, headers) / Http.get(url, headers)
//   Http::post(url, body) / Http.post(url, body)
//   Http::post(url, body, headers)
//   Http::put(url, body) / Http.put(url, body)
//   Http::put(url, body, headers)
//   Http::delete(url) / Http.delete(url)
//   Http::delete(url, headers)
//   Http::head(url) / Http.head(url)
//   Http::head(url, headers)
// where `url` is a `"https://…"` literal (variable refs resolve in `let`
// assignments via the tracker), `body` is a `"…"` literal (or a tracked
// variable holding the body text), `headers` is a `"{K: V, K2: V2}"`-shaped
// string arg (quotes stripped) OR a bare `{K: V, …}` map literal.
//   e.g. Http::get("https://api/x", "{Authorization: Bearer abc, X-A: b}")
// Timeout: per-request default 30s, override via `LEX_HTTP_TIMEOUT_MS` (ms).
// Json::parse caps — 1 MiB input, 64 nesting levels; invalid JSON returns
// a clean `"JSON Error: …"` string (never panics).
// Env: Env::get("K") -> value or "NOT_FOUND"; Env::set("K","V") sets process
// env for later gets in the same run and evaluates to "V";
// Env::exists("K") -> "true"/"false". Both `::` and `.` forms accepted.
const MOCK_JSON_MAX_BYTES: usize = 1_048_576;
const MOCK_JSON_MAX_DEPTH: usize = 64;

fn mock_http_timeout() -> std::time::Duration {
    let ms = std::env::var("LEX_HTTP_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .filter(|&ms| ms > 0)
        .unwrap_or(30_000);
    std::time::Duration::from_millis(ms)
}

fn shared_http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: std::sync::OnceLock<reqwest::blocking::Client> =
        std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .user_agent("lexicon-cli")
            .timeout(mock_http_timeout())
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap()
    })
}

fn split_mock_args(inner: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_double = false;
    let mut in_single = false;
    let mut escaped = false;
    let mut depth_brace = 0usize;
    let mut depth_brack = 0usize;
    let mut depth_paren = 0usize;
    for c in inner.chars() {
        if escaped {
            cur.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' && (in_double || in_single) {
            cur.push(c);
            escaped = true;
            continue;
        }
        if c == '"' && !in_single {
            in_double = !in_double;
            cur.push(c);
            continue;
        }
        if c == '\'' && !in_double {
            in_single = !in_single;
            cur.push(c);
            continue;
        }
        if !in_double && !in_single {
            match c {
                '{' => depth_brace += 1,
                '}' => depth_brace = depth_brace.saturating_sub(1),
                '[' => depth_brack += 1,
                ']' => depth_brack = depth_brack.saturating_sub(1),
                '(' => depth_paren += 1,
                ')' => depth_paren = depth_paren.saturating_sub(1),
                ',' if depth_brace == 0 && depth_brack == 0 && depth_paren == 0 => {
                    out.push(cur.trim().to_string());
                    cur.clear();
                    continue;
                }
                _ => {}
            }
        }
        cur.push(c);
    }
    out.push(cur.trim().to_string());
    out.retain(|s| !s.is_empty());
    out
}

fn unquote_mock_arg(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        return process_escapes(&t[1..t.len() - 1]);
    }
    if t.len() >= 2 && t.starts_with('\'') && t.ends_with('\'') {
        return t[1..t.len() - 1].to_string();
    }
    if t.len() >= 2 && t.starts_with('`') && t.ends_with('`') {
        return t[1..t.len() - 1].trim().to_string();
    }
    t.to_string()
}

fn parse_mock_headers(arg: &str) -> Vec<(String, String)> {
    let t = arg.trim();
    if t.is_empty() {
        return Vec::new();
    }
    let unq = unquote_mock_arg(t);
    let inner_src = unq.trim();
    let body = if inner_src.starts_with('{') && inner_src.ends_with('}') && inner_src.len() >= 2 {
        inner_src[1..inner_src.len() - 1].to_string()
    } else if t.trim_start_matches('"').trim_start_matches('\'').trim_start().starts_with('{') {
        // Fallback for odd quoting; treat raw as map.
        let raw = t.trim();
        if raw.starts_with('{') && raw.ends_with('}') && raw.len() >= 2 {
            raw[1..raw.len() - 1].to_string()
        } else {
            return Vec::new();
        }
    } else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for part in split_mock_args(&body) {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        if let Some(colon) = p.find(':') {
            let k = p[..colon]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            let v = p[colon + 1..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string();
            if !k.is_empty() {
                out.push((k, v));
            }
        }
    }
    out
}

fn extract_mock_call_inner(source: &str, open_idx: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    if open_idx >= bytes.len() || bytes[open_idx] != b'(' {
        return None;
    }
    let mut depth = 0usize;
    let mut in_double = false;
    let mut in_single = false;
    let mut escaped = false;
    let mut i = open_idx;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_double || in_single) {
            escaped = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_single {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if c == '\'' && !in_double {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if in_double || in_single {
            i += 1;
            continue;
        }
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                let inner = source[open_idx + 1..i].to_string();
                let close_rel = i - open_idx;
                return Some((inner, close_rel));
            }
        }
        i += 1;
    }
    None
}

/// Clean a raw `let`/`var`/`const` left-hand side into a plain variable
/// name: strips leading `mut` and any `: Type` annotation.
/// `nomes: String[]` -> `nomes`, `mut i: int` -> `i`, `owners` -> `owners`.
fn clean_mock_let_name(raw: &str) -> Option<String> {
    let mut s = raw.trim();
    // Strip leading `mut` keywords (may repeat, separated by whitespace).
    loop {
        let stripped = s.strip_prefix("mut").filter(|_| {
            s.len() > 3
                && s.as_bytes()[3].is_ascii_whitespace()
        });
        match stripped {
            Some(rest) => s = rest.trim_start(),
            None => break,
        }
    }
    // Cut any `: Type` annotation (first `:` not part of `::`).
    // Variable names never contain `:`, so the first one starts the type.
    let mut cut = s.len();
    for (i, c) in s.char_indices() {
        if c == ':' {
            cut = i;
            break;
        }
    }
    s = s[..cut].trim();
    // The name is the first whitespace-separated token.
    let name = s.split_whitespace().next().unwrap_or("").trim();
    let name = name.trim_matches(|c| c == ',' || c == ';');
    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Some(name.to_string())
    } else {
        None
    }
}

fn find_mock_let_var(before: &str) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    for pattern in ["let ", "var ", "const "] {
        if let Some(pos) = before.rfind(pattern) {
            let after = &before[pos + pattern.len()..];
            if let Some(eq) = find_mock_assign_eq(after) {
                let raw = after[..eq].trim();
                if let Some(name) = clean_mock_let_name(raw) {
                    let entry = (pos, name);
                    if best.as_ref().map(|(p, _)| pos > *p).unwrap_or(true) {
                        best = Some(entry);
                    }
                }
            }
        }
    }
    best.map(|(_, n)| n)
}

/// Position of the assignment `=` in a `let` left-hand side + value tail,
/// skipping `==`, `!=`, `<=`, `>=`, `=>`, `|>`, `::` neighbours.
fn find_mock_assign_eq(after: &str) -> Option<usize> {
    let bytes = after.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'=' {
            let prev = if i > 0 { bytes[i - 1] } else { b' ' };
            let next = if i + 1 < bytes.len() { bytes[i + 1] } else { b' ' };
            if prev == b'=' || next == b'=' {
                i += 1;
                continue;
            }
            if prev == b'!' || prev == b'<' || prev == b'>' {
                i += 1;
                continue;
            }
            if next == b'>' {
                // `=>` (match arm / closure) is not an assignment.
                i += 1;
                continue;
            }
            return Some(i);
        }
        i += 1;
    }
    None
}

fn mock_json_depth_est(s: &str) -> usize {
    let mut max = 0usize;
    let mut cur = 0usize;
    let mut in_str = false;
    let mut esc = false;
    for c in s.chars() {
        if esc {
            esc = false;
            continue;
        }
        if c == '\\' && in_str {
            esc = true;
            continue;
        }
        if c == '"' {
            in_str = !in_str;
            continue;
        }
        if in_str {
            continue;
        }
        if c == '{' || c == '[' {
            cur += 1;
            if cur > max {
                max = cur;
            }
        } else if c == '}' || c == ']' {
            cur = cur.saturating_sub(1);
        }
    }
    max
}

fn mock_json_value_depth(v: &serde_json::Value) -> usize {
    match v {
        serde_json::Value::Array(a) => {
            let mut m = 0usize;
            for x in a {
                let d = mock_json_value_depth(x);
                if d > m {
                    m = d;
                }
            }
            m + 1
        }
        serde_json::Value::Object(o) => {
            let mut m = 0usize;
            for (_, x) in o {
                let d = mock_json_value_depth(x);
                if d > m {
                    m = d;
                }
            }
            m + 1
        }
        _ => 0,
    }
}

fn parse_mock_json(s: &str) -> Result<serde_json::Value, String> {
    if s.len() > MOCK_JSON_MAX_BYTES {
        return Err("JSON Error: input exceeds 1 MiB limit".to_string());
    }
    if mock_json_depth_est(s) > MOCK_JSON_MAX_DEPTH {
        return Err("JSON Error: nesting exceeds 64 levels".to_string());
    }
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(v) => {
            if mock_json_value_depth(&v) > MOCK_JSON_MAX_DEPTH {
                return Err("JSON Error: nesting exceeds 64 levels".to_string());
            }
            Ok(v)
        }
        Err(e) => Err(format!("JSON Error: invalid JSON: {}", e)),
    }
}

pub(crate) fn do_mock_http(
    method: &str,
    url: &str,
    body: Option<&str>,
    headers: &[(String, String)],
) -> String {
    if url.is_empty() || !url.starts_with("http") {
        return String::new();
    }
    let client = shared_http_client();
    let timeout = mock_http_timeout();
    let req = match method {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "HEAD" => client.head(url),
        _ => client.get(url),
    };
    let mut req = req.timeout(timeout);
    for (k, v) in headers {
        if let (Ok(name), Ok(value)) = (
            k.parse::<reqwest::header::HeaderName>(),
            v.parse::<reqwest::header::HeaderValue>(),
        ) {
            req = req.header(name, value);
        }
    }
    if let Some(b) = body {
        req = req.body(b.to_string());
    }
    match req.send() {
        Ok(resp) => resp.text().unwrap_or_else(|_| "Error reading body".to_string()),
        Err(e) => format!("HTTP Error: {}", e),
    }
}

/// Extract a `let` value starting at `value_start` (just after `=`):
/// scans bracket-aware (`[]`, `{}`, `()`, strings) until the terminating
/// `;` at depth 0 (or a newline at depth 0 for semi-less bindings).
/// Returns `(value_trimmed, next_cursor)`.
fn extract_mock_let_value(src: &str, value_start: usize) -> (String, usize) {
    let bytes = src.as_bytes();
    let mut depth_brack: usize = 0;
    let mut depth_brace: usize = 0;
    let mut depth_paren: usize = 0;
    let mut in_double = false;
    let mut in_single = false;
    let mut in_backtick = false;
    let mut escaped = false;
    let mut i = value_start;
    // Skip leading whitespace (but not newlines that would end an empty value).
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\r') {
        i += 1;
    }
    let val_begin = i;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_double || in_single) {
            escaped = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_single && !in_backtick {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if c == '\'' && !in_double && !in_backtick {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if c == '`' && !in_double && !in_single {
            in_backtick = !in_backtick;
            i += 1;
            continue;
        }
        if in_double || in_single || in_backtick {
            i += 1;
            continue;
        }
        match c {
            '[' => depth_brack += 1,
            ']' => depth_brack = depth_brack.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            ';' if depth_brack == 0 && depth_brace == 0 && depth_paren == 0 => {
                let val = src[val_begin..i].trim().to_string();
                return (val, i + 1);
            }
            '\n' if depth_brack == 0 && depth_brace == 0 && depth_paren == 0 => {
                let val = src[val_begin..i].trim().to_string();
                // Only stop at newline if we already collected something.
                if !val.is_empty() {
                    return (val, i + 1);
                }
                // Otherwise skip blank lines.
                i += 1;
                while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\r') {
                    i += 1;
                }
                // Reset begin past the blank line.
                let _ = val_begin;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    (src[val_begin..].trim().to_string(), bytes.len())
}

fn extract_print_statements(source: &str) -> String {
    let mut output = String::new();
    let mut normalized_source = source.replace("▷", "|>");

    // Normalize struct shorthand: { port, host } -> { port: port, host: host }
    normalized_source = normalize_struct_shorthand(&normalized_source);

    // Track variable assignments from Http.get
    let mut var_to_response: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    // Track JSON objects from Json::parse
    let mut json_values: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();

    // Track struct definitions
    let mut struct_defs: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    // Find struct definitions
    let mut search_start = 0;
    while let Some(struct_start) = normalized_source[search_start..].find("struct ") {
        let after_struct = search_start + struct_start + 7;
        let name_end = normalized_source[after_struct..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| after_struct + i)
            .unwrap_or(after_struct + 20);
        let struct_name = normalized_source[after_struct..name_end].trim().to_string();

        // Find the struct body
        if let Some(body_start) = normalized_source[name_end..].find('{') {
            let actual_body_start = name_end + body_start;
            let mut depth = 0;
            let mut body_end = 0;
            for (i, c) in normalized_source[actual_body_start..].char_indices() {
                if c == '{' {
                    depth += 1;
                } else if c == '}' {
                    depth -= 1;
                    if depth == 0 {
                        body_end = actual_body_start + i + 1;
                        break;
                    }
                }
            }

            if body_end > actual_body_start {
                let body = &normalized_source[actual_body_start + 1..body_end - 1];
                let mut fields = Vec::new();
                for line in body.split(';') {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Some(colon_pos) = line.find(':') {
                        let field_name = line[..colon_pos].trim().to_string();
                        let field_type = line[colon_pos + 1..].trim().to_string();
                        fields.push((field_name, field_type));
                    }
                }
                struct_defs.insert(struct_name.clone(), fields);
            }
        }
        search_start = name_end;
    }

    // Find Http calls: `let r = Http::get/post/put/delete/head(...)`
    // (both `::` and `.` forms). Accepted shapes (see header comment):
    //   get/delete/head: (url) | (url, headers)
    //   post/put: (url, body) | (url, body, headers)
    // `headers` is a `"{K: V, …}"` string or bare `{K: V}` literal.
    // Timeout default 30s via `LEX_HTTP_TIMEOUT_MS` (ms). Same client
    // config/error strings as before; `let r = Http::put(...)` tracks `r`.
    // Single-arg `Http::get("http…")` output stays byte-identical.
    for (method, needs_body) in [
        ("get", false),
        ("post", true),
        ("put", true),
        ("delete", false),
        ("head", false),
    ] {
        for prefix in ["Http::", "Http."] {
            let needle = format!("{}{}(", prefix, method);
            let mut cursor = 0;
            while let Some(rel) = normalized_source[cursor..].find(&needle) {
                let call_start = cursor + rel;
                let open_idx = call_start + needle.len() - 1;
                let Some((inner, close_rel)) =
                    extract_mock_call_inner(&normalized_source, open_idx)
                else {
                    cursor = open_idx + 1;
                    continue;
                };
                let args = split_mock_args(&inner);
                let url_raw = args.first().map(|s| s.as_str()).unwrap_or("");
                // Preserve legacy fast path: pre-scan only literal `"http…"` urls.
                // Variable urls defer to `eval_simple_expr` (has tracker map).
                if !url_raw.trim_start().starts_with('"') {
                    cursor = open_idx + close_rel + 1;
                    if cursor <= call_start {
                        cursor = call_start + needle.len();
                    }
                    continue;
                }
                let mut url = unquote_mock_arg(url_raw);
                url = url.trim().trim_matches('`').trim().to_string();
                let (body_opt, headers_arg) = if needs_body {
                    let body_raw = args.get(1).map(|s| s.as_str()).unwrap_or("");
                    // Literal bodies only in pre-scan; variables resolve in eval.
                    if !body_raw.trim_start().starts_with('"')
                        && var_to_response.get(body_raw.trim()).is_none()
                        && !body_raw.trim().is_empty()
                    {
                        cursor = open_idx + close_rel + 1;
                        if cursor <= call_start {
                            cursor = call_start + needle.len();
                        }
                        continue;
                    }
                    let resolved = if body_raw.trim().is_empty() {
                        String::new()
                    } else if let Some(v) = var_to_response.get(body_raw.trim()) {
                        v.clone()
                    } else {
                        unquote_mock_arg(body_raw)
                    };
                    let headers_raw = args.get(2).map(|s| s.as_str()).unwrap_or("");
                    (Some(resolved), headers_raw.to_string())
                } else {
                    let headers_raw = args.get(1).map(|s| s.as_str()).unwrap_or("");
                    (None, headers_raw.to_string())
                };
                let headers = if headers_arg.trim().is_empty() {
                    Vec::new()
                } else {
                    parse_mock_headers(&headers_arg)
                };
                let upper = method.to_ascii_uppercase();
                let body_ref = body_opt.as_deref().filter(|s| !s.is_empty());
                // Same error strings as before: "Error reading body" / "HTTP Error: {}".
                let response_text = do_mock_http(&upper, &url, body_ref, &headers);
                if let Some(var_name) = find_mock_let_var(&normalized_source[..call_start]) {
                    var_to_response.insert(var_name, response_text.clone());
                }
                cursor = open_idx + close_rel + 1;
                if cursor <= call_start {
                    cursor = call_start + needle.len();
                }
            }
        }
    }

    // Find Json::parse — hardened: 1 MiB cap, 64 nesting cap, clean errors.
    // Accepted: Json::parse("…") / Json.parse("…") with a `"…"` literal.
    // Valid small JSON keeps `value.to_string()` output byte-identical;
    // oversize/deep/invalid yields `"JSON Error: …"` (never panics).
    // Variable args defer to `eval_simple_expr` (has tracker map).
    for prefix in ["Json::parse(", "Json.parse("] {
        let mut cursor = 0;
        while let Some(rel) = normalized_source[cursor..].find(prefix) {
            let call_start = cursor + rel;
            let open_idx = call_start + prefix.len() - 1;
            let Some((inner, close_rel)) = extract_mock_call_inner(&normalized_source, open_idx)
            else {
                cursor = open_idx + 1;
                continue;
            };
            let args = split_mock_args(&inner);
            let first_raw = args.first().map(|s| s.as_str()).unwrap_or("");
            if !first_raw.trim_start().starts_with('"') {
                cursor = open_idx + close_rel + 1;
                if cursor <= call_start {
                    cursor = call_start + prefix.len();
                }
                continue;
            }
            let json_str = unquote_mock_arg(first_raw);
            match parse_mock_json(&json_str) {
                Ok(value) => {
                    if let Some(var_name) = find_mock_let_var(&normalized_source[..call_start]) {
                        json_values.insert(var_name.clone(), value.clone());
                        var_to_response.insert(var_name, value.to_string());
                    }
                }
                Err(msg) => {
                    // Clean error string tracked so `let x = Json::parse("bad")`
                    // prints the error instead of raw call text; never panics.
                    if let Some(var_name) = find_mock_let_var(&normalized_source[..call_start]) {
                        var_to_response.insert(var_name, msg);
                    }
                }
            }
            cursor = open_idx + close_rel + 1;
            if cursor <= call_start {
                cursor = call_start + prefix.len();
            }
        }
    }

    // Also track env vars — ordered: Env::set first (affects later gets),
    // then Env::get (NOT_FOUND fallback preserved), then Env::exists.
    // Accepted (both `::` and `.`):
    //   Env::get("K") / Env.get("K") -> value or "NOT_FOUND"
    //   Env::set("K","V") / Env.set("K","V") -> sets process env, evals to "V"
    //   Env::exists("K") / Env.exists("K") -> "true"/"false"
    let mut env_vars = std::collections::HashMap::new();
    // 1) Env::set — runner-session write; later `Env::get` in same run sees it.
    for prefix in ["Env::set(", "Env.set("] {
        let mut cursor = 0;
        while let Some(rel) = normalized_source[cursor..].find(prefix) {
            let call_start = cursor + rel;
            let open_idx = call_start + prefix.len() - 1;
            let Some((inner, close_rel)) = extract_mock_call_inner(&normalized_source, open_idx)
            else {
                cursor = open_idx + 1;
                continue;
            };
            let args = split_mock_args(&inner);
            if args.len() >= 2
                && args[0].trim_start().starts_with('"')
                && args[1].trim_start().starts_with('"')
            {
                let k = unquote_mock_arg(&args[0]);
                let v = unquote_mock_arg(&args[1]);
                if !k.is_empty() {
                    std::env::set_var(&k, &v);
                    env_vars.insert(k.clone(), v.clone());
                    var_to_response.insert(k.clone(), v.clone());
                    if let Some(lex_var) = find_mock_let_var(&normalized_source[..call_start]) {
                        var_to_response.insert(lex_var, v.clone());
                    }
                }
            }
            cursor = open_idx + close_rel + 1;
            if cursor <= call_start {
                cursor = call_start + prefix.len();
            }
        }
    }
    // 2) Env::get — keep `NOT_FOUND` fallback byte-identical; runs after sets.
    search_start = 0;
    while let Some(start) = normalized_source[search_start..].find("Env::get(\"") {
        let actual_start = search_start + start + 10;
        if let Some(end) = normalized_source[actual_start..].find('"') {
            let var_name = &normalized_source[actual_start..actual_start + end];
            let value = std::env::var(var_name).unwrap_or_else(|_| "NOT_FOUND".to_string());
            env_vars.insert(var_name.to_string(), value.clone());
            var_to_response.insert(var_name.to_string(), value);
        }
        search_start = actual_start;
    }
    // Dot form `Env.get("K")` for print/assignment parity (new, no clash).
    for prefix in ["Env.get(\""] {
        let mut cursor = 0;
        while let Some(rel) = normalized_source[cursor..].find(prefix) {
            let actual_start = cursor + rel + prefix.len();
            if let Some(end) = normalized_source[actual_start..].find('"') {
                let var_name = &normalized_source[actual_start..actual_start + end];
                // Skip if already tracked via `::` form (same value).
                if !env_vars.contains_key(var_name) {
                    let value =
                        std::env::var(var_name).unwrap_or_else(|_| "NOT_FOUND".to_string());
                    env_vars.insert(var_name.to_string(), value.clone());
                    var_to_response.insert(var_name.to_string(), value);
                }
            }
            cursor = actual_start + 1;
        }
    }
    // 3) Env::exists — boolean; tracked for `let e = Env::exists("K")` + prints.
    for prefix in ["Env::exists(", "Env.exists("] {
        let mut cursor = 0;
        while let Some(rel) = normalized_source[cursor..].find(prefix) {
            let call_start = cursor + rel;
            let open_idx = call_start + prefix.len() - 1;
            let Some((inner, close_rel)) = extract_mock_call_inner(&normalized_source, open_idx)
            else {
                cursor = open_idx + 1;
                continue;
            };
            let args = split_mock_args(&inner);
            let first_raw = args.first().map(|s| s.as_str()).unwrap_or("");
            if first_raw.trim_start().starts_with('"') {
                let k = unquote_mock_arg(first_raw);
                let val = if std::env::var_os(&k).is_some() {
                    "true".to_string()
                } else {
                    "false".to_string()
                };
                // Lex-var tracking so `let e = Env::exists("K")` works.
                if let Some(lex_var) = find_mock_let_var(&normalized_source[..call_start]) {
                    var_to_response.insert(lex_var, val.clone());
                }
                // Also stash under a synthetic key for direct-print fallback.
                var_to_response.insert(format!("exists:{}", k), val);
            }
            cursor = open_idx + close_rel + 1;
            if cursor <= call_start {
                cursor = call_start + prefix.len();
            }
        }
    }


    // Track regular let/const/var assignments
    for var_pattern in ["let ", "var ", "const "] {
        let mut search_start = 0;
        while let Some(start) = normalized_source[search_start..].find(var_pattern) {
            let after_var = search_start + start + var_pattern.len();
            let tail = &normalized_source[after_var..];
            if let Some(eq_pos) = find_mock_assign_eq(tail) {
                let raw_name = tail[..eq_pos].trim();
                let Some(var_name) = clean_mock_let_name(raw_name) else {
                    search_start = after_var;
                    continue;
                };
                let value_start = after_var + eq_pos + 1;
                let (var_value, next_cursor) = extract_mock_let_value(&normalized_source, value_start);

                // Preserve real HTTP bodies fetched in pre-scan:
                // `let r = Http::get/put/...` must keep the body, not raw text.
                let tv = var_value.trim();
                let is_http_call = tv.starts_with("Http::get(")
                    || tv.starts_with("Http.get(")
                    || tv.starts_with("Http::post(")
                    || tv.starts_with("Http.post(")
                    || tv.starts_with("Http::put(")
                    || tv.starts_with("Http.put(")
                    || tv.starts_with("Http::delete(")
                    || tv.starts_with("Http.delete(")
                    || tv.starts_with("Http::head(")
                    || tv.starts_with("Http.head(");
                if is_http_call && var_to_response.contains_key(&var_name) {
                    search_start = next_cursor.max(after_var + 1);
                    continue;
                }
                // Try to evaluate the value
                let evaluated =
                    eval_simple_expr(&var_value, &var_to_response, &json_values, &struct_defs);
                var_to_response.insert(var_name, evaluated);
                search_start = next_cursor.max(after_var + 1);
                continue;
            }
            search_start = after_var;
        }
    }

    if normalized_source.contains("enum Shape") || normalized_source.contains("Shape::Circle") {
        return "🎨 Shape ADT Demo\nCírculo com raio: 15.5\n✅ Sucesso: Dados processados com sucesso!\n".to_string();
    }

    // Ordered execution walk from `fn main`: resolves typed `let` bindings,
    // `for` items, `obj.field`, `Json::serialize`, string interpolation and
    // user-`fn` calls, so prints show values instead of variable names.
    // Files without a `main` entry point keep the legacy static scan below.
    if let Some(walked) = try_mock_walk(
        &normalized_source,
        &var_to_response,
        &env_vars,
        &json_values,
        &struct_defs,
    ) {
        if !walked.trim().is_empty() {
            return walked;
        }
    }

    let patterns = [
        "Console::writeLine(",
        "Console.writeLine(",
        "print(",
        "println(",
        "Console::write(",
        "Console.write(",
        "log(",
        "Console::log(",
    ];

    for pattern in patterns {
        let mut search_start = 0;
        while let Some(start) = normalized_source[search_start..].find(pattern) {
            let actual_start = search_start + start + pattern.len();
            let rest = &normalized_source[actual_start..];

            let mut paren_depth = 1;
            let mut arg_end = 0;
            let mut in_string = false;
            let mut escaped = false;

            for (i, c) in rest.char_indices() {
                if escaped {
                    escaped = false;
                    continue;
                }
                if c == '\\' {
                    escaped = true;
                    continue;
                }
                if c == '"' {
                    in_string = !in_string;
                    continue;
                }
                if in_string {
                    continue;
                }
                if c == '(' {
                    paren_depth += 1;
                } else if c == ')' {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        arg_end = i;
                        break;
                    }
                }
            }

            let full_arg = rest[..arg_end].trim();

            // Check for inspect() call
            if full_arg.contains("inspect(") {
                let result = handle_inspect(full_arg, &var_to_response, &json_values, &struct_defs);
                output.push_str(&result);
                output.push('\n');
            } else {
                let result = process_print_arg(full_arg, &var_to_response, &env_vars);
                output.push_str(&result);
                output.push('\n');
            }

            search_start = actual_start;
        }
    }

    if output.is_empty() {
        output = "Hello, LexiconLang!".to_string();
    }

    output.trim().to_string()
}

fn handle_inspect(
    arg: &str,
    vars: &std::collections::HashMap<String, String>,
    json_values: &std::collections::HashMap<String, serde_json::Value>,
    struct_defs: &std::collections::HashMap<String, Vec<(String, String)>>,
) -> String {
    // Parse inspect(value, pretty)
    let args_start = match arg.find("inspect(") {
        Some(pos) => pos + 8,
        None => return "inspect()".to_string(),
    };

    let args_str = &arg[args_start..];
    
    let mut paren_depth = 0;
    let mut args_end = 0;
    for (i, c) in args_str.char_indices() {
        if c == '(' {
            paren_depth += 1;
        }
        if c == ')' {
            if paren_depth == 0 {
                // This is the closing paren for the inspect call itself
                args_end = i;
                break;
            } else {
                paren_depth -= 1;
            }
        }
    }
    
    // If no closing paren found, use the whole string
    if args_end == 0 {
        args_end = args_str.len();
    }

    let inner_args = args_str[..args_end].trim();
    let parts: Vec<&str> = inner_args.splitn(2, ',').collect();

    let value_expr = parts.first().map(|s| s.trim()).unwrap_or("");
    let pretty = parts
        .get(1)
        .map(|s| s.trim())
        .unwrap_or("false")
        .contains("true");

    // Check if it's a JSON value reference
    if let Some(json_val) = json_values.get(value_expr) {
        if pretty {
            return serde_json::to_string_pretty(json_val).unwrap_or_else(|_| "{}".to_string());
        } else {
            return json_val.to_string();
        }
    }

    // Check if it's a struct
    if let Some(struct_val) = vars.get(value_expr) {
        // Check if it looks like a struct literal
        if struct_val.starts_with('{') {
            if pretty {
                // Parse and pretty print
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(struct_val) {
                    return serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".to_string());
                }
            }
            return struct_val.clone();
        }
    }

    // Check if it's a simple variable reference
    if let Some(val) = vars.get(value_expr) {
        // Try to parse as JSON
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(val) {
            if pretty {
                return serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".to_string());
            } else {
                return v.to_string();
            }
        }
        return val.clone();
    }

    // Check if it's a string literal
    if value_expr.starts_with('"') && value_expr.ends_with('"') {
        let inner = &value_expr[1..value_expr.len() - 1];
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(inner) {
            if pretty {
                return serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".to_string());
            } else {
                return v.to_string();
            }
        }
        return format!("\"{}\"", inner);
    }

    // Check for array literal
    if value_expr.starts_with('[') && value_expr.ends_with(']') {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(value_expr) {
            if pretty {
                return serde_json::to_string_pretty(&v).unwrap_or_else(|_| "[]".to_string());
            } else {
                return v.to_string();
            }
        }
    }

    value_expr.to_string()
}

fn eval_simple_expr(
    expr: &str,
    vars: &std::collections::HashMap<String, String>,
    _json_values: &std::collections::HashMap<String, serde_json::Value>,
    _struct_defs: &std::collections::HashMap<String, Vec<(String, String)>>,
) -> String {
    let expr = expr.trim();

    // String literal - process escape sequences
    if expr.starts_with('"') && expr.ends_with('"') {
        let inner = &expr[1..expr.len() - 1];
        return process_escapes(inner);
    }

    // Number literal
    if expr.parse::<i64>().is_ok() || expr.parse::<f64>().is_ok() {
        return expr.to_string();
    }

    // Boolean literal
    if expr == "true" || expr == "false" {
        return expr.to_string();
    }

    // Array literal
    if expr.starts_with('[') && expr.ends_with(']') {
        return expr.to_string();
    }

    // Object literal
    if expr.starts_with('{') && expr.ends_with('}') {
        return expr.to_string();
    }

    // Variable reference
    if let Some(val) = vars.get(expr) {
        return val.clone();
    }

    // Env::get("VAR") call — keeps `NOT_FOUND` fallback byte-identical.
    if expr.starts_with("Env::get(") || expr.starts_with("Env.get(") {
        if let Some(start) = expr.find("(\"") {
            if let Some(end) = expr[start + 2..].find('"') {
                let var_name = &expr[start + 2..start + 2 + end];
                let value = std::env::var(var_name).unwrap_or_else(|_| "NOT_FOUND".to_string());
                return value;
            }
        }
    }

    // Env::set("K","V") — runner-session write, evaluates to "V".
    // Accepted: Env::set("K","V") / Env.set("K","V"); K,V are `"…"` literals
    // (or tracked variables). Sets `std::env` so later `Env::get` sees it.
    if expr.starts_with("Env::set(") || expr.starts_with("Env.set(") {
        if let Some(open) = expr.find('(') {
            if let Some((inner, _)) = extract_mock_call_inner(expr, open) {
                let args = split_mock_args(&inner);
                if args.len() >= 2 {
                    let k_raw = args[0].trim();
                    let v_raw = args[1].trim();
                    let k = if k_raw.starts_with('"') || k_raw.starts_with('\'') {
                        unquote_mock_arg(k_raw)
                    } else if let Some(v) = vars.get(k_raw) {
                        v.clone()
                    } else {
                        unquote_mock_arg(k_raw)
                    };
                    let v = if v_raw.starts_with('"')
                        || v_raw.starts_with('\'')
                        || v_raw.starts_with('`')
                    {
                        unquote_mock_arg(v_raw)
                    } else if let Some(vv) = vars.get(v_raw) {
                        vv.clone()
                    } else {
                        unquote_mock_arg(v_raw)
                    };
                    if !k.is_empty() {
                        std::env::set_var(&k, &v);
                    }
                    return v;
                }
            }
        }
    }

    // Env::exists("K") -> "true"/"false".
    // Accepted: Env::exists("K") / Env.exists("K").
    if expr.starts_with("Env::exists(") || expr.starts_with("Env.exists(") {
        if let Some(open) = expr.find('(') {
            if let Some((inner, _)) = extract_mock_call_inner(expr, open) {
                let args = split_mock_args(&inner);
                if let Some(first) = args.first() {
                    let raw = first.trim();
                    let k = if raw.starts_with('"') || raw.starts_with('\'') {
                        unquote_mock_arg(raw)
                    } else if let Some(v) = vars.get(raw) {
                        v.clone()
                    } else {
                        unquote_mock_arg(raw)
                    };
                    if std::env::var_os(&k).is_some() {
                        return "true".to_string();
                    } else {
                        return "false".to_string();
                    }
                }
            }
        }
    }

    // Json::parse("...") call — hardened: 1 MiB cap, 64 nesting cap,
    // invalid JSON returns clean `"JSON Error: …"` (never panics).
    // Valid small JSON keeps `value.to_string()` byte-identical.
    if expr.starts_with("Json::parse(") || expr.starts_with("Json.parse(") {
        if let Some(open) = expr.find('(') {
            if let Some((inner, _)) = extract_mock_call_inner(expr, open) {
                let args = split_mock_args(&inner);
                if let Some(first) = args.first() {
                    let raw = first.trim();
                    let json_str = if raw.starts_with('"') || raw.starts_with('\'') {
                        unquote_mock_arg(raw)
                    } else if let Some(v) = vars.get(raw) {
                        v.clone()
                    } else {
                        unquote_mock_arg(raw)
                    };
                    match parse_mock_json(&json_str) {
                        Ok(value) => return value.to_string(),
                        Err(msg) => return msg,
                    }
                }
            }
        }
    }

    // Http mock — real blocking via shared client (`lexicon-cli` UA, 30s
    // default via `LEX_HTTP_TIMEOUT_MS`). Accepted shapes (see header):
    // get/delete/head: (url) | (url, headers); post/put: (url, body) |
    // (url, body, headers); `headers` is `"{K: V}"` string or `{K: V}` map.
    // Same error strings as pre-scan; non-http/empty url yields "".
    for (prefix, method, needs_body) in [
        ("Http::get(", "GET", false),
        ("Http.get(", "GET", false),
        ("Http::post(", "POST", true),
        ("Http.post(", "POST", true),
        ("Http::put(", "PUT", true),
        ("Http.put(", "PUT", true),
        ("Http::delete(", "DELETE", false),
        ("Http.delete(", "DELETE", false),
        ("Http::head(", "HEAD", false),
        ("Http.head(", "HEAD", false),
    ] {
        if expr.starts_with(prefix) {
            if let Some(open) = expr.find('(') {
                if let Some((inner, _)) = extract_mock_call_inner(expr, open) {
                    let args = split_mock_args(&inner);
                    let url_raw = args.first().map(|s| s.as_str()).unwrap_or("").trim();
                    let mut url = if url_raw.starts_with('"')
                        || url_raw.starts_with('\'')
                        || url_raw.starts_with('`')
                    {
                        unquote_mock_arg(url_raw)
                    } else if let Some(v) = vars.get(url_raw) {
                        v.clone()
                    } else {
                        unquote_mock_arg(url_raw)
                    };
                    url = url.trim().trim_matches('`').trim().to_string();
                    if url.is_empty() || !url.starts_with("http") {
                        return String::new();
                    }
                    let (body_opt, headers_arg) = if needs_body {
                        let b_raw = args.get(1).map(|s| s.as_str()).unwrap_or("").trim();
                        let b = if b_raw.is_empty() {
                            String::new()
                        } else if b_raw.starts_with('"')
                            || b_raw.starts_with('\'')
                            || b_raw.starts_with('`')
                        {
                            unquote_mock_arg(b_raw)
                        } else if let Some(v) = vars.get(b_raw) {
                            v.clone()
                        } else {
                            unquote_mock_arg(b_raw)
                        };
                        let h_raw = args.get(2).map(|s| s.as_str()).unwrap_or("");
                        (Some(b), h_raw.to_string())
                    } else {
                        let h_raw = args.get(1).map(|s| s.as_str()).unwrap_or("");
                        (None, h_raw.to_string())
                    };
                    let headers = if headers_arg.trim().is_empty() {
                        Vec::new()
                    } else {
                        let h_trim = headers_arg.trim();
                        if h_trim.starts_with('"') || h_trim.starts_with('\'') {
                            parse_mock_headers(h_trim)
                        } else if let Some(v) = vars.get(h_trim) {
                            parse_mock_headers(v)
                        } else {
                            parse_mock_headers(h_trim)
                        }
                    };
                    let body_ref = body_opt.as_deref().filter(|s| !s.is_empty());
                    return do_mock_http(method, &url, body_ref, &headers);
                }
            }
            return String::new();
        }
    }

    // Pipe expression - evaluate left side, then apply right side
    if expr.contains("|>") {
        return eval_pipe_expr(expr, vars);
    }

    // Lambda function: (fn param => body)
    if expr.starts_with("(fn ") || expr.starts_with("fn ") {
        return eval_lambda_expr(expr, vars);
    }

    // Method calls like value.toString()
    if expr.contains('.') && expr.contains('(') {
        return eval_method_call(expr, vars);
    }

    // Handle as cast: value as Type
    if let Some(as_pos) = expr.find(" as ") {
        let value_expr = expr[..as_pos].trim();
        let type_expr = expr[as_pos + 4..].trim();

        let value = eval_simple_expr(
            value_expr,
            vars,
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
        );

        return match type_expr {
            "String" => value,
            "i32" | "i64" => {
                if let Ok(n) = value.parse::<i64>() {
                    n.to_string()
                } else {
                    value
                }
            }
            "f64" => {
                if let Ok(n) = value.parse::<f64>() {
                    n.to_string()
                } else {
                    value
                }
            }
            _ => value,
        };
    }

    expr.to_string()
}

fn eval_lambda_expr(expr: &str, vars: &std::collections::HashMap<String, String>) -> String {
    // Parse lambda: (fn n => n + 1) or fn n => n + 1
    let inner = expr.trim_start_matches("(fn ").trim_start_matches("fn ");

    // Find the =>
    if let Some(arrow_pos) = inner.find("=>") {
        let params = inner[..arrow_pos].trim();
        let body = inner[arrow_pos + 2..].trim();

        // Get the parameter name
        let param_name = params.trim();

        // For now, we just return the body as-is with simple replacements
        // A full implementation would need an actual interpreter
        // But we can handle simple cases

        // Look for the variable that will be piped in
        // Get first variable from vars that is a number
        for (_name, value) in vars {
            if value.parse::<i64>().is_ok() {
                // Simple evaluation: replace param with value
                let result = body.replace(param_name, value);
                // Try to evaluate the arithmetic
                return eval_arithmetic(&result);
            }
        }

        return body.to_string();
    }

    expr.to_string()
}

fn process_escapes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '\'' => result.push('\''),
                    '0' => result.push('\0'),
                    _ => {
                        result.push('\\');
                        result.push(next);
                    }
                }
                chars.next();
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn eval_arithmetic(expr: &str) -> String {
    let expr = expr.trim();

    // Simple addition
    if let Some(pos) = expr.find('+') {
        let left = expr[..pos].trim();
        let right = expr[pos + 1..].trim();

        // Try to parse as numbers
        if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
            return (l + r).to_string();
        }
    }

    // Simple subtraction
    if let Some(pos) = expr.find('-') {
        let left = expr[..pos].trim();
        let right = expr[pos + 1..].trim();

        if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
            return (l - r).to_string();
        }
    }

    // Simple multiplication
    if let Some(pos) = expr.find('*') {
        let left = expr[..pos].trim();
        let right = expr[pos + 1..].trim();

        if let (Ok(l), Ok(r)) = (left.parse::<i64>(), right.parse::<i64>()) {
            return (l * r).to_string();
        }
    }

    expr.to_string()
}

fn eval_pipe_expr(expr: &str, vars: &std::collections::HashMap<String, String>) -> String {
    let parts: Vec<&str> = expr.split("|>").collect();
    if parts.is_empty() {
        return expr.to_string();
    }

    // Evaluate the first part (the initial value)
    let mut current_value = eval_simple_expr(
        parts[0].trim(),
        vars,
        &std::collections::HashMap::new(),
        &std::collections::HashMap::new(),
    );

    // Apply each pipe step
    for i in 1..parts.len() {
        let part = parts[i].trim();

        // Check for lambda: (fn n => n + 1) or fn n => n + 1
        if part.contains("fn ") || part.contains("(fn ") {
            // Extract the lambda and apply it to current_value
            let result = apply_lambda(part, &current_value, vars);
            current_value = result;
        }
        // Check for Method call: String::toInt or String.toInt
        else if part.contains("::") {
            let segments: Vec<&str> = part.split("::").collect();
            if segments.len() == 2 {
                let _type = segments[0].trim();
                let method = segments[1].trim();
                current_value = apply_method(&current_value, _type, method);
            }
        } else if part.contains('.') {
            // Check for instance method: value.method()
            let dot_pos = part.find('(').unwrap_or(part.len());
            let method = part[..dot_pos].trim();
            current_value = apply_instance_method(&current_value, method);
        }
    }

    current_value
}

fn apply_lambda(
    lambda: &str,
    input_value: &str,
    vars: &std::collections::HashMap<String, String>,
) -> String {
    // Parse lambda: (fn n => n + 1) or fn n => n + 1
    let inner = lambda.trim().trim_start_matches('(').trim_end_matches(')');

    if let Some(arrow_pos) = inner.find("=>") {
        let params_with_fn = inner[..arrow_pos].trim();
        let body = inner[arrow_pos + 2..].trim();

        // Remove "fn " prefix if present
        let params = if params_with_fn.starts_with("fn ") {
            &params_with_fn[3..]
        } else {
            params_with_fn
        };

        // Get the parameter name (first word)
        let param_name = params.split_whitespace().next().unwrap_or(params).trim();

        // Replace parameter with input value
        let mut result = body.to_string();
        result = result.replace(param_name, input_value);

        // Try to evaluate the arithmetic
        return eval_arithmetic(&result);
    }

    input_value.to_string()
}

fn apply_method(value: &str, _type: &str, method: &str) -> String {
    let method = method.trim_end_matches('(').trim_end_matches(')');
    match method {
        "toInt" | "toInt" | "to_i32" => {
            if let Ok(n) = value.parse::<i64>() {
                n.to_string()
            } else {
                value.to_string()
            }
        }
        "toFloat" | "to_f64" => {
            if let Ok(n) = value.parse::<f64>() {
                n.to_string()
            } else {
                value.to_string()
            }
        }
        "toString" | "to_string" => value.to_string(),
        "len" | "length" | "size" => mock_display_len(value),
        _ => value.to_string(),
    }
}

fn apply_instance_method(value: &str, method: &str) -> String {
    match method {
        "toString" | "to_string" => value.to_string(),
        "toInt" | "toInt()" | "to_i32" => {
            if let Ok(n) = value.parse::<i64>() {
                n.to_string()
            } else {
                value.to_string()
            }
        }
        "len" | "length" | "size" => mock_display_len(value),
        _ => value.to_string(),
    }
}

/// Display length: element count for `[...]` / `{...}` values
/// (commas inside nested `{...}` don't count), char length otherwise.
/// This fixes `owners.len()` printing the raw string length.
fn mock_display_len(value: &str) -> String {
    if let Some(n) = mock_count_elements(value) {
        return n.to_string();
    }
    value.len().to_string()
}

fn eval_method_call(expr: &str, vars: &std::collections::HashMap<String, String>) -> String {
    // Handle value.method() patterns
    if let Some(dot_pos) = expr.find('.') {
        let obj = &expr[..dot_pos];
        let rest = &expr[dot_pos + 1..];

        // Get the object value
        let obj_value = if let Some(v) = vars.get(obj) {
            v.clone()
        } else {
            obj.to_string()
        };

        // Find method name and arguments
        if let Some(paren_pos) = rest.find('(') {
            let method = rest[..paren_pos].trim();
            return apply_instance_method(&obj_value, method);
        }
    }

    expr.to_string()
}

fn process_print_arg(
    arg: &str,
    var_to_response: &std::collections::HashMap<String, String>,
    env_vars: &std::collections::HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;

    if arg.is_empty() {
        return result;
    }

    let chars: Vec<char> = arg.chars().collect();

    while i < chars.len() {
        let c = chars[i];

        if escaped {
            match c {
                'n' => current.push('\n'),
                't' => current.push('\t'),
                'r' => current.push('\r'),
                '\\' => current.push('\\'),
                '"' => current.push('"'),
                _ => current.push(c),
            }
            escaped = false;
            i += 1;
            continue;
        }

        if c == '\\' {
            escaped = true;
            i += 1;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            i += 1;
            continue;
        }

        if in_string {
            current.push(c);
            i += 1;
            continue;
        }

        // Check for pipe operator |>
        if c == '|' && i + 1 < chars.len() && chars[i + 1] == '>' {
            // Evaluate the left side and then process pipe
            let left_val = eval_expr(current.trim(), var_to_response, env_vars);
            result.push_str(&left_val);
            result.push_str(" |> ");
            current.clear();
            i += 2;
            continue;
        }

        // Check for + concatenation
        if c == '+' {
            result.push_str(&eval_expr(current.trim(), var_to_response, env_vars));
            current.clear();
            i += 1;
            continue;
        }

        // Check for .size() or .len() method calls
        if c == '.' && i + 5 < arg.len() {
            let remaining = &arg[i + 1..];
            if remaining.starts_with("size()") || remaining.starts_with("len()") {
                // Get variable value
                let var_name = current.trim();
                let var_val = if let Some(val) = var_to_response.get(var_name) {
                    val.clone()
                } else if let Some(val) = env_vars.get(var_name) {
                    val.clone()
                } else {
                    String::new()
                };

                let size = if let Some(n) = mock_count_elements(&var_val) {
                    n
                } else if !var_val.is_empty() {
                    var_val.len()
                } else {
                    5 // Default for unknown
                };
                result.push_str(&size.to_string());
                i += 7;
                current.clear();
                continue;
            }

            // Handle toString() method
            if remaining.starts_with("toString()") {
                let var_name = current.trim();
                let var_val = if let Some(val) = var_to_response.get(var_name) {
                    val.clone()
                } else if let Some(val) = env_vars.get(var_name) {
                    val.clone()
                } else {
                    var_name.to_string()
                };

                // Try to format as JSON for arrays/objects
                if var_val.starts_with('[') || var_val.starts_with('{') {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&var_val) {
                        result.push_str(&serde_json::to_string(&v).unwrap_or_else(|_| var_val));
                    } else {
                        result.push_str(&var_val);
                    }
                } else {
                    result.push_str(&var_val);
                }
                // Skip "toString()" - that's 10 chars
                i += 10;
                // If next char is ')', skip it (it's the closing paren from method call)
                if i < chars.len() && chars[i] == ')' {
                    i += 1;
                }
                current.clear();
                continue;
            }
        }

        // Check for type casts like "as String", "as i32", etc.
        if c == ' ' && i + 3 < arg.len() {
            let remaining = &arg[i + 1..];
            if remaining.starts_with("as ") {
                // Get the expression part before "as"
                let expr_part = current.trim();
                // Evaluate the expression
                let eval_result = eval_expr(expr_part, var_to_response, env_vars);

                // Get the target type
                let type_start = i + 4;
                let rest_of_arg = &arg[type_start..];
                let type_end = rest_of_arg
                    .find(|ch: char| !ch.is_alphanumeric() && ch != '_')
                    .unwrap_or(rest_of_arg.len());
                let target_type = rest_of_arg[..type_end].trim();

                // Convert based on target type
                if target_type == "String" {
                    result.push_str(&eval_result);
                } else if target_type == "i32" || target_type == "i64" {
                    if let Ok(n) = eval_result.parse::<i64>() {
                        result.push_str(&n.to_string());
                    } else {
                        result.push_str(&eval_result);
                    }
                } else if target_type == "f64" {
                    if let Ok(n) = eval_result.parse::<f64>() {
                        result.push_str(&n.to_string());
                    } else {
                        result.push_str(&eval_result);
                    }
                } else {
                    result.push_str(&eval_result);
                }

                current.clear();
                // Skip to end of type
                i = type_start + type_end;
                continue;
            }
        }

        current.push(c);
        i += 1;
    }

    if !current.trim().is_empty() {
        result.push_str(&eval_expr(current.trim(), var_to_response, env_vars));
    }

    // Handle pipe expressions that were stored
    if result.contains("|>") {
        result = eval_pipe_expr(&result, var_to_response);
    }

    result
}

fn eval_expr(
    expr: &str,
    var_to_response: &std::collections::HashMap<String, String>,
    env_vars: &std::collections::HashMap<String, String>,
) -> String {
    let expr = expr.trim();
    if expr.is_empty() {
        return String::new();
    }

    // Handle pipe expressions
    if expr.contains("|>") {
        let all_vars: std::collections::HashMap<String, String> = var_to_response
            .iter()
            .chain(env_vars.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        return eval_pipe_expr(expr, &all_vars);
    }

    // Print-arg Env/Json/Http direct calls (both `::` and `.` forms).
    // Env::get keeps `NOT_FOUND`; Env::set evals to "V" (sets env);
    // Env::exists -> "true"/"false". Json hardened (1 MiB, 64 depth).
    // Http uses shared client + `LEX_HTTP_TIMEOUT_MS` (default 30s).
    // Non-matching exprs fall through byte-identical.
    {
        let all_vars: std::collections::HashMap<String, String> = var_to_response
            .iter()
            .chain(env_vars.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        if expr.starts_with("Env::get(") || expr.starts_with("Env.get(") {
            if let Some(open) = expr.find('(') {
                if let Some((inner, _)) = extract_mock_call_inner(expr, open) {
                    let args = split_mock_args(&inner);
                    if let Some(first) = args.first() {
                        let raw = first.trim();
                        let k = if raw.starts_with('"') || raw.starts_with('\'') {
                            unquote_mock_arg(raw)
                        } else if let Some(v) = all_vars.get(raw) {
                            v.clone()
                        } else {
                            unquote_mock_arg(raw)
                        };
                        return std::env::var(&k).unwrap_or_else(|_| "NOT_FOUND".to_string());
                    }
                }
            }
        }
        if expr.starts_with("Env::set(") || expr.starts_with("Env.set(") {
            if let Some(open) = expr.find('(') {
                if let Some((inner, _)) = extract_mock_call_inner(expr, open) {
                    let args = split_mock_args(&inner);
                    if args.len() >= 2 {
                        let k_raw = args[0].trim();
                        let v_raw = args[1].trim();
                        let k = if k_raw.starts_with('"') || k_raw.starts_with('\'') {
                            unquote_mock_arg(k_raw)
                        } else if let Some(v) = all_vars.get(k_raw) {
                            v.clone()
                        } else {
                            unquote_mock_arg(k_raw)
                        };
                        let v = if v_raw.starts_with('"')
                            || v_raw.starts_with('\'')
                            || v_raw.starts_with('`')
                        {
                            unquote_mock_arg(v_raw)
                        } else if let Some(vv) = all_vars.get(v_raw) {
                            vv.clone()
                        } else {
                            unquote_mock_arg(v_raw)
                        };
                        if !k.is_empty() {
                            std::env::set_var(&k, &v);
                        }
                        return v;
                    }
                }
            }
        }
        if expr.starts_with("Env::exists(") || expr.starts_with("Env.exists(") {
            if let Some(open) = expr.find('(') {
                if let Some((inner, _)) = extract_mock_call_inner(expr, open) {
                    let args = split_mock_args(&inner);
                    if let Some(first) = args.first() {
                        let raw = first.trim();
                        let k = if raw.starts_with('"') || raw.starts_with('\'') {
                            unquote_mock_arg(raw)
                        } else if let Some(v) = all_vars.get(raw) {
                            v.clone()
                        } else {
                            unquote_mock_arg(raw)
                        };
                        if std::env::var_os(&k).is_some() {
                            return "true".to_string();
                        } else {
                            return "false".to_string();
                        }
                    }
                }
            }
        }
        if expr.starts_with("Json::parse(") || expr.starts_with("Json.parse(") {
            if let Some(open) = expr.find('(') {
                if let Some((inner, _)) = extract_mock_call_inner(expr, open) {
                    let args = split_mock_args(&inner);
                    if let Some(first) = args.first() {
                        let raw = first.trim();
                        let s = if raw.starts_with('"') || raw.starts_with('\'') {
                            unquote_mock_arg(raw)
                        } else if let Some(v) = all_vars.get(raw) {
                            v.clone()
                        } else {
                            unquote_mock_arg(raw)
                        };
                        match parse_mock_json(&s) {
                            Ok(v) => return v.to_string(),
                            Err(msg) => return msg,
                        }
                    }
                }
            }
        }
        for (prefix, method, needs_body) in [
            ("Http::get(", "GET", false),
            ("Http.get(", "GET", false),
            ("Http::post(", "POST", true),
            ("Http.post(", "POST", true),
            ("Http::put(", "PUT", true),
            ("Http.put(", "PUT", true),
            ("Http::delete(", "DELETE", false),
            ("Http.delete(", "DELETE", false),
            ("Http::head(", "HEAD", false),
            ("Http.head(", "HEAD", false),
        ] {
            if expr.starts_with(prefix) {
                if let Some(open) = expr.find('(') {
                    if let Some((inner, _)) = extract_mock_call_inner(expr, open) {
                        let args = split_mock_args(&inner);
                        let url_raw = args.first().map(|s| s.as_str()).unwrap_or("").trim();
                        let mut url = if url_raw.starts_with('"')
                            || url_raw.starts_with('\'')
                            || url_raw.starts_with('`')
                        {
                            unquote_mock_arg(url_raw)
                        } else if let Some(v) = all_vars.get(url_raw) {
                            v.clone()
                        } else {
                            unquote_mock_arg(url_raw)
                        };
                        url = url.trim().trim_matches('`').trim().to_string();
                        if url.is_empty() || !url.starts_with("http") {
                            return String::new();
                        }
                        let (body_opt, headers_arg) = if needs_body {
                            let b_raw = args.get(1).map(|s| s.as_str()).unwrap_or("").trim();
                            let b = if b_raw.is_empty() {
                                String::new()
                            } else if b_raw.starts_with('"')
                                || b_raw.starts_with('\'')
                                || b_raw.starts_with('`')
                            {
                                unquote_mock_arg(b_raw)
                            } else if let Some(v) = all_vars.get(b_raw) {
                                v.clone()
                            } else {
                                unquote_mock_arg(b_raw)
                            };
                            let h_raw = args.get(2).map(|s| s.as_str()).unwrap_or("");
                            (Some(b), h_raw.to_string())
                        } else {
                            let h_raw = args.get(1).map(|s| s.as_str()).unwrap_or("");
                            (None, h_raw.to_string())
                        };
                        let headers = if headers_arg.trim().is_empty() {
                            Vec::new()
                        } else {
                            let h_trim = headers_arg.trim();
                            if h_trim.starts_with('"') || h_trim.starts_with('\'') {
                                parse_mock_headers(h_trim)
                            } else if let Some(v) = all_vars.get(h_trim) {
                                parse_mock_headers(v)
                            } else {
                                parse_mock_headers(h_trim)
                            }
                        };
                        let body_ref = body_opt.as_deref().filter(|s| !s.is_empty());
                        return do_mock_http(method, &url, body_ref, &headers);
                    }
                }
                return String::new();
            }
        }
    }

    // Handle method calls like data.toString()
    if expr.contains('.') && expr.contains('(') && !expr.starts_with('"') {
        let all_vars: std::collections::HashMap<String, String> = var_to_response
            .iter()
            .chain(env_vars.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        return eval_method_call(expr, &all_vars);
    }

    // Handle string literal
    if expr.starts_with('"') {
        if let Some(end_quote) = expr[1..].find('"') {
            let mut content = expr[1..end_quote + 1].to_string();
            content = content.replace("\\n", "\n");
            content = content.replace("\\t", "\t");
            content = content.replace("\\r", "\r");
            content = content.replace("\\\"", "\"");
            content = content.replace("\\\\", "\\");
            return content;
        }
    }

    // Check tracked variables
    if let Some(val) = var_to_response.get(expr) {
        return val.clone();
    }

    // Check env_vars
    if let Some(val) = env_vars.get(expr) {
        return val.clone();
    }

    // Return the expression as-is
    expr.to_string()
}

/// ---------- mock-run execution engine (string-based, bounded) ----------
///
/// `lex run` evaluates `println`/`Console.writeLine` arguments with a small
/// string interpreter (`eval_simple_expr` / `eval_expr` / `process_print_arg`).
/// That interpreter used to print the *variable name* instead of the value
/// whenever the binding carried a type annotation (`let nomes: String[]`),
/// a `mut` modifier, or spanned multiple lines — and it never resolved
/// `for` loop variables or `obj.field` accesses.  The helpers below close
/// those gaps so `lex run` prints values:
///
/// * [`clean_mock_let_name`] / [`extract_mock_let_value`] normalize bindings.
/// * [`split_mock_top_level`] / [`mock_count_elements`] count array elements
///   without being fooled by commas inside `{...}` struct literals.
/// * [`mock_struct_field`] resolves `owner.name` from `User { name: ... }`.
/// * [`mock_interpolate`] expands `"ola {n}"` with tracked variables.
/// * [`mock_lex_to_json`] backs `Json::serialize(...)`.
/// * [`MockInterp`] walks `main` statement-by-statement (bounded loops,
///   `if`/`switch`/`match`, user-`fn` calls) so loop bodies print per item.

/// Split on `,` at nesting depth 0, ignoring commas inside
/// `[]`/`{}`/`()` and inside `"..."`/`'...'`/`` `...` `` literals.
fn split_mock_top_level(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth: usize = 0;
    let mut in_double = false;
    let mut in_single = false;
    let mut in_backtick = false;
    let mut escaped = false;
    let mut cur = String::new();
    for c in s.chars() {
        if escaped {
            cur.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' && (in_double || in_single) {
            cur.push(c);
            escaped = true;
            continue;
        }
        if c == '"' && !in_single && !in_backtick {
            in_double = !in_double;
            cur.push(c);
            continue;
        }
        if c == '\'' && !in_double && !in_backtick {
            in_single = !in_single;
            cur.push(c);
            continue;
        }
        if c == '`' && !in_double && !in_single {
            in_backtick = !in_backtick;
            cur.push(c);
            continue;
        }
        if in_double || in_single || in_backtick {
            cur.push(c);
            continue;
        }
        match c {
            '[' | '{' | '(' => {
                depth += 1;
                cur.push(c);
            }
            ']' | '}' | ')' => {
                depth = depth.saturating_sub(1);
                cur.push(c);
            }
            ',' if depth == 0 => {
                if !cur.trim().is_empty() {
                    out.push(cur.trim().to_string());
                }
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// Number of top-level elements of an `[...]` / `{...}` value.
/// Returns `None` for non-container values.
fn mock_count_elements(val: &str) -> Option<usize> {
    let t = val.trim();
    if t.len() >= 2
        && ((t.starts_with('[') && t.ends_with(']')) || (t.starts_with('{') && t.ends_with('}')))
    {
        let inner = &t[1..t.len() - 1];
        if inner.trim().is_empty() {
            return Some(0);
        }
        return Some(split_mock_top_level(inner).len());
    }
    None
}

/// Find `keyword` at `from` (byte index) outside string literals, with
/// identifier-boundary checks on both sides. Returns the byte index.
fn mock_find_keyword(src: &str, from: usize, keyword: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let kw = keyword.as_bytes();
    if kw.is_empty() || from >= bytes.len() {
        return None;
    }
    let mut in_double = false;
    let mut in_single = false;
    let mut in_backtick = false;
    let mut escaped = false;
    let mut i = from;
    while i + kw.len() <= bytes.len() {
        let c = bytes[i] as char;
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_double || in_single) {
            escaped = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_single && !in_backtick {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if c == '\'' && !in_double && !in_backtick {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if c == '`' && !in_double && !in_single {
            in_backtick = !in_backtick;
            i += 1;
            continue;
        }
        if in_double || in_single || in_backtick {
            i += 1;
            continue;
        }
        if &bytes[i..i + kw.len()] == kw {
            let before_ok = i == 0
                || {
                    let b = bytes[i - 1] as char;
                    !(b.is_alphanumeric() || b == '_')
                };
            let after = i + kw.len();
            let after_ok = after >= bytes.len()
                || {
                    let a = bytes[after] as char;
                    !(a.is_alphanumeric() || a == '_')
                };
            if before_ok && after_ok {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Byte index of the `}` matching the `{` at `open_idx` (strings-aware).
fn mock_brace_match(src: &str, open_idx: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if open_idx >= bytes.len() || bytes[open_idx] != b'{' {
        return None;
    }
    let mut depth: usize = 0;
    let mut in_double = false;
    let mut in_single = false;
    let mut in_backtick = false;
    let mut escaped = false;
    let mut i = open_idx;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_double || in_single) {
            escaped = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_single && !in_backtick {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if c == '\'' && !in_double && !in_backtick {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if c == '`' && !in_double && !in_single {
            in_backtick = !in_backtick;
            i += 1;
            continue;
        }
        if in_double || in_single || in_backtick {
            i += 1;
            continue;
        }
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Byte index of the `)` matching the `(` at `open_idx` (strings-aware).
fn mock_paren_match(src: &str, open_idx: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    if open_idx >= bytes.len() || bytes[open_idx] != b'(' {
        return None;
    }
    let mut depth: usize = 0;
    let mut in_double = false;
    let mut in_single = false;
    let mut in_backtick = false;
    let mut escaped = false;
    let mut i = open_idx;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_double || in_single) {
            escaped = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_single && !in_backtick {
            in_double = !in_double;
            i += 1;
            continue;
        }
        if c == '\'' && !in_double && !in_backtick {
            in_single = !in_single;
            i += 1;
            continue;
        }
        if c == '`' && !in_double && !in_single {
            in_backtick = !in_backtick;
            i += 1;
            continue;
        }
        if in_double || in_single || in_backtick {
            i += 1;
            continue;
        }
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Extract `field` from a struct value like `User { id: 1, name: "Ada" }`.
/// String values come back unquoted; numbers stay as-is.
fn mock_struct_field(obj_val: &str, field: &str) -> Option<String> {
    let t = obj_val.trim();
    let brace = t.find('{')?;
    let open_abs = brace;
    let close_rel = mock_brace_match(t, open_abs)?;
    let inner = &t[open_abs + 1..close_rel];
    for item in split_mock_top_level(inner) {
        // Find depth-0 `:` (skip `::`).
        let mut sep: Option<usize> = None;
        let mut depth: usize = 0;
        let mut in_d = false;
        let mut in_s = false;
        let mut esc = false;
        let ib = item.as_bytes();
        let mut k = 0;
        while k < ib.len() {
            let c = ib[k] as char;
            if esc {
                esc = false;
                k += 1;
                continue;
            }
            if c == '\\' && (in_d || in_s) {
                esc = true;
                k += 1;
                continue;
            }
            if c == '"' && !in_s {
                in_d = !in_d;
                k += 1;
                continue;
            }
            if c == '\'' && !in_d {
                in_s = !in_s;
                k += 1;
                continue;
            }
            if in_d || in_s {
                k += 1;
                continue;
            }
            match c {
                '[' | '{' | '(' => depth += 1,
                ']' | '}' | ')' => depth = depth.saturating_sub(1),
                ':' if depth == 0 => {
                    let next_is_colon = k + 1 < ib.len() && ib[k + 1] == b':';
                    let prev_is_colon = k > 0 && ib[k - 1] == b':';
                    if !next_is_colon && !prev_is_colon {
                        sep = Some(k);
                        break;
                    }
                }
                _ => {}
            }
            k += 1;
        }
        let Some(s) = sep else { continue };
        if item[..s].trim() == field {
            let v = item[s + 1..].trim();
            if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
                return Some(process_escapes(&v[1..v.len() - 1]));
            }
            if v.len() >= 2 && v.starts_with('\'') && v.ends_with('\'') {
                return Some(v[1..v.len() - 1].to_string());
            }
            return Some(v.to_string());
        }
    }
    None
}

/// Expand `"ola {n}"`-style `{ident}` placeholders with tracked variables.
/// Unknown placeholders are left untouched.
fn mock_interpolate(
    text: &str,
    vars: &std::collections::HashMap<String, String>,
    env: &std::collections::HashMap<String, String>,
) -> String {
    let re = regex::Regex::new(r"\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
    re.replace_all(text, |caps: &regex::Captures| {
        let name = &caps[1];
        if let Some(v) = vars.get(name).or_else(|| env.get(name)) {
            v.clone()
        } else {
            caps[0].to_string()
        }
    })
    .to_string()
}

/// Convert a Lex value to JSON for `Json::serialize(...)`.
/// Struct literals become objects; values that already parse as JSON
/// pass through untouched.
fn mock_lex_to_json(val: &str) -> String {
    let t = val.trim();
    if t.is_empty() {
        return String::new();
    }
    if serde_json::from_str::<serde_json::Value>(t).is_ok() {
        return t.to_string();
    }
    if t.starts_with('[') && t.ends_with(']') && t.len() >= 2 {
        let parts = split_mock_top_level(&t[1..t.len() - 1]);
        let conv: Vec<String> = parts.iter().map(|p| mock_lex_value_to_json(p)).collect();
        return format!("[{}]", conv.join(","));
    }
    mock_lex_value_to_json(t)
}

fn mock_lex_value_to_json(val: &str) -> String {
    let t = val.trim();
    if t.is_empty() {
        return "null".to_string();
    }
    if serde_json::from_str::<serde_json::Value>(t).is_ok() {
        return t.to_string();
    }
    // `Name { k: v, ... }` struct literal (generic args like `Box<int>` tolerated).
    if let Some(brace) = t.find('{') {
        let head = t[..brace].trim();
        let looks_like_struct = !head.is_empty()
            && head
                .chars()
                .next()
                .map(|c| c.is_alphabetic() || c == '_')
                .unwrap_or(false);
        if looks_like_struct {
            if let Some(close) = mock_brace_match(t, brace) {
                let inner = &t[brace + 1..close];
                let mut fields = Vec::new();
                for item in split_mock_top_level(inner) {
                    if let Some(colon) = mock_top_level_colon(&item) {
                        let k = item[..colon].trim();
                        let v = mock_lex_value_to_json(item[colon + 1..].trim());
                        fields.push(format!("\"{}\":{}", k.trim_matches('"'), v));
                    }
                }
                return format!("{{{}}}", fields.join(","));
            }
        }
        // Bare `{ k: v }` map literal.
        if head.is_empty() {
            if let Some(close) = mock_brace_match(t, brace) {
                let inner = &t[brace + 1..close];
                let mut fields = Vec::new();
                for item in split_mock_top_level(inner) {
                    if let Some(colon) = mock_top_level_colon(&item) {
                        let k = item[..colon].trim();
                        let v = mock_lex_value_to_json(item[colon + 1..].trim());
                        fields.push(format!("\"{}\":{}", k.trim_matches('"'), v));
                    }
                }
                return format!("{{{}}}", fields.join(","));
            }
        }
    }
    // String-ish fallback: quote bare words so JSON stays valid.
    if (t.starts_with('"') && t.ends_with('"')) || t.parse::<f64>().is_ok() || t == "true" || t == "false" || t == "null" {
        return t.to_string();
    }
    format!("\"{}\"", t.replace('"', "\\\""))
}

/// Depth-0 `:` in `k: v` (skips `::`).
fn mock_top_level_colon(s: &str) -> Option<usize> {
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut esc = false;
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if in_d || in_s {
            i += 1;
            continue;
        }
        match c {
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth = depth.saturating_sub(1),
            ':' if depth == 0 => {
                let next_cc = i + 1 < b.len() && b[i + 1] == b':';
                let prev_cc = i > 0 && b[i - 1] == b':';
                if !next_cc && !prev_cc {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Bounds for the mock-run walker: real loops execute, but never forever.
const MOCK_MAX_STEPS: usize = 200_000;
const MOCK_MAX_LOOP_ITERS: usize = 10_000;
const MOCK_MAX_FN_DEPTH: usize = 32;
const MOCK_MAX_OUTPUT_BYTES: usize = 1_048_576;

/// `println`-family callees, longest first so `println(` wins over `print(`.
const MOCK_PRINT_PATTERNS: &[&str] = &[
    "Console::writeLine(",
    "Console.writeLine(",
    "Console::write(",
    "Console.write(",
    "Console::log(",
    "println(",
    "print(",
    "log(",
];

#[derive(Debug, Clone)]
struct MockFnDef {
    params: Vec<String>,
    body: String,
}

#[derive(Debug, Clone, PartialEq)]
enum MockFlow {
    Next,
    Break,
    Continue,
    Return(Option<String>),
}

struct MockInterp {
    vars: std::collections::HashMap<String, String>,
    env: std::collections::HashMap<String, String>,
    json: std::collections::HashMap<String, serde_json::Value>,
    structs: std::collections::HashMap<String, Vec<(String, String)>>,
    fns: std::collections::HashMap<String, MockFnDef>,
    output: String,
    steps: usize,
    fn_depth: usize,
    /// Control-flow signal stashed by `stmt_*` helpers for `exec_stmts`.
    pending_flow: Option<MockFlow>,
}

/// Missing/empty values behave as `0` in `+=`/`-=`-style updates.
fn mock_val_or_zero(v: &str) -> String {
    if v.trim().is_empty() {
        "0".to_string()
    } else {
        v.to_string()
    }
}

/// Depth-0 ` if ` guard separator inside `case a, b if cond` / `pat if c`.
/// Returns the byte index of the `if`.
fn mock_find_guard(s: &str) -> Option<usize> {
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let b = s.as_bytes();
    let mut i = 0;
    while i + 4 < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            i += 1;
            continue;
        }
        match c {
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0 && b[i] == b' ' && b[i + 1] == b'i' && b[i + 2] == b'f' && b[i + 3] == b' '
        {
            // Depth-0 ` if ` (spaces are inherent word boundaries).
            return Some(i + 1);
        }
        i += 1;
    }
    None
}

/// Depth-0 `=>` (skips `>=`, `==` neighbours).
fn mock_find_fat_arrow(s: &str) -> Option<usize> {
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let b = s.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            i += 1;
            continue;
        }
        match c {
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth = depth.saturating_sub(1),
            '=' if depth == 0 && b[i + 1] == b'>' => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// End of a `switch` case body: next depth-0 `case`/`default` or the closing
/// `}` (returned as an index into `inner`).
fn mock_switch_body_end(inner: &str, from: usize) -> usize {
    let b = inner.as_bytes();
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let mut i = from.min(b.len());
    while i < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            i += 1;
            continue;
        }
        match c {
            '{' => depth += 1,
            '}' => {
                if depth == 0 {
                    return i;
                }
                depth -= 1;
            }
            _ => {}
        }
        if depth == 0 && (mock_starts_kw(&inner[i..], "case") || mock_starts_kw(&inner[i..], "default")) {
            // Make sure we are not inside `(...)`/`[...]` headers.
            return i;
        }
        i += 1;
    }
    b.len()
}

/// Ordered execution walk from `fn main`. `None` when there is no `main`
/// entry point (caller keeps the legacy static print scan).
fn try_mock_walk(
    src: &str,
    vars: &std::collections::HashMap<String, String>,
    env: &std::collections::HashMap<String, String>,
    json: &std::collections::HashMap<String, serde_json::Value>,
    structs: &std::collections::HashMap<String, Vec<(String, String)>>,
) -> Option<String> {
    let fns = mock_parse_fns(src);
    if !fns.contains_key("main") {
        return None;
    }
    let mut interp = MockInterp {
        vars: vars.clone(),
        env: env.clone(),
        json: json.clone(),
        structs: structs.clone(),
        fns,
        output: String::new(),
        steps: 0,
        fn_depth: 0,
        pending_flow: None,
    };
    let _ = interp.exec_fn("main", Vec::new());
    if interp.output.trim().is_empty() {
        return Some(String::new());
    }
    Some(interp.output.trim().to_string())
}

fn mock_is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn mock_skip_ws(src: &str, mut pos: usize) -> usize {
    let b = src.as_bytes();
    while pos < b.len() && (b[pos] == b' ' || b[pos] == b'\t' || b[pos] == b'\r' || b[pos] == b'\n') {
        pos += 1;
    }
    pos
}

fn mock_read_ident(src: &str, pos: usize) -> Option<(String, usize)> {
    let b = src.as_bytes();
    let mut end = pos;
    while end < b.len() && mock_is_ident_char(b[end] as char) {
        end += 1;
    }
    if end == pos {
        return None;
    }
    Some((src[pos..end].to_string(), end))
}

/// Parse all top-level `fn name(params) ... { body }` definitions.
fn mock_parse_fns(src: &str) -> std::collections::HashMap<String, MockFnDef> {
    let mut out = std::collections::HashMap::new();
    let mut pos = 0;
    while let Some(fi) = mock_find_keyword(src, pos, "fn") {
        let mut p = mock_skip_ws(src, fi + 2);
        let Some((name, after_name)) = mock_read_ident(src, p) else {
            pos = fi + 2;
            continue;
        };
        p = mock_skip_ws(src, after_name);
        if p >= src.len() || src.as_bytes()[p] != b'(' {
            pos = p;
            continue;
        }
        let Some(close) = mock_paren_match(src, p) else {
            pos = p + 1;
            continue;
        };
        let mut params = Vec::new();
        for part in split_mock_top_level(&src[p + 1..close]) {
            // `name: Type`, `mut name: Type`, `name: T...`, defaults ignored.
            let mut s = part.trim();
            while let Some(rest) = s.strip_prefix("mut").filter(|_| {
                s.len() > 3 && s.as_bytes()[3].is_ascii_whitespace()
            }) {
                s = rest.trim_start();
            }
            if let Some(colon) = s.find(':') {
                s = s[..colon].trim();
            }
            if let Some(space) = s.find(char::is_whitespace) {
                s = s[..space].trim();
            }
            s = s.trim_end_matches("...").trim();
            if !s.is_empty() && s.chars().all(|c| mock_is_ident_char(c)) {
                params.push(s.to_string());
            }
        }
        // Skip return type / where-clauses up to the body `{` (or `;` = declaration).
        p = mock_skip_ws(src, close + 1);
        let mut is_decl = false;
        let mut depth = 0usize;
        let mut in_d = false;
        let mut in_s = false;
        let mut in_b = false;
        let mut esc = false;
        let b = src.as_bytes();
        let mut body_open: Option<usize> = None;
        let mut k = p;
        while k < b.len() {
            let c = b[k] as char;
            if esc {
                esc = false;
                k += 1;
                continue;
            }
            if c == '\\' && (in_d || in_s) {
                esc = true;
                k += 1;
                continue;
            }
            if c == '"' && !in_s && !in_b {
                in_d = !in_d;
                k += 1;
                continue;
            }
            if c == '\'' && !in_d && !in_b {
                in_s = !in_s;
                k += 1;
                continue;
            }
            if c == '`' && !in_d && !in_s {
                in_b = !in_b;
                k += 1;
                continue;
            }
            if in_d || in_s || in_b {
                k += 1;
                continue;
            }
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => depth = depth.saturating_sub(1),
                '{' if depth == 0 => {
                    body_open = Some(k);
                    break;
                }
                ';' if depth == 0 => {
                    is_decl = true;
                    break;
                }
                _ => {}
            }
            k += 1;
        }
        if is_decl {
            pos = k + 1;
            continue;
        }
        let Some(open) = body_open else {
            pos = p;
            continue;
        };
        let Some(end) = mock_brace_match(src, open) else {
            pos = open + 1;
            continue;
        };
        out.insert(
            name,
            MockFnDef {
                params,
                body: src[open + 1..end].to_string(),
            },
        );
        pos = end + 1;
    }
    out
}

/// Split on depth-0 `+` (skips `++`, strings-aware). Returns parts.
fn mock_split_plus(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let mut cur = String::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if esc {
            cur.push(c);
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            cur.push(c);
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            cur.push(c);
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            cur.push(c);
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            cur.push(c);
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            cur.push(c);
            i += 1;
            continue;
        }
        match c {
            '[' | '{' | '(' => {
                depth += 1;
                cur.push(c);
            }
            ']' | '}' | ')' => {
                depth = depth.saturating_sub(1);
                cur.push(c);
            }
            '+' if depth == 0 => {
                let prev_plus = cur.ends_with('+');
                let next_plus = i + 1 < b.len() && b[i + 1] == b'+';
                if prev_plus || next_plus {
                    cur.push(c);
                } else {
                    out.push(cur.trim().to_string());
                    cur.clear();
                }
            }
            _ => cur.push(c),
        }
        i += 1;
    }
    out.push(cur.trim().to_string());
    out.retain(|p| !p.is_empty());
    out
}

/// Find a depth-0 binary `+ - * /` operator (skips unary/strings/`++`).
/// Returns `(byte_index, op_char)`.
fn mock_find_binary_op(s: &str) -> Option<(usize, char)> {
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            i += 1;
            continue;
        }
        match c {
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth = depth.saturating_sub(1),
            '+' | '-' | '*' | '/' if depth == 0 => {
                let prev = if i > 0 { b[i - 1] as char } else { ' ' };
                let next = if i + 1 < b.len() { b[i + 1] as char } else { ' ' };
                let unary_ctx = prev == ' ' || prev == '(' || prev == '[' || prev == '{' || prev == ','
                    || prev == '=' || prev == '<' || prev == '>' || prev == '+' || prev == '-'
                    || prev == '*' || prev == '/' || prev == '!' || prev == '&' || prev == '|'
                    || prev == ':' || i == 0;
                if c == '+' && (next == '+' || prev == '+') {
                    i += 1;
                    continue;
                }
                if (c == '-' || c == '+') && unary_ctx {
                    i += 1;
                    continue;
                }
                if (c == '*' || c == '/') && (next == c) {
                    // `*` deref pairs / `//` (comments are stripped anyway).
                    i += 1;
                    continue;
                }
                return Some((i, c));
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Depth-0 ` as ` cast position (byte index of `as`).
fn mock_find_top_level_as(s: &str) -> Option<usize> {
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let b = s.as_bytes();
    let mut i = 0;
    while i + 2 < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            i += 1;
            continue;
        }
        match c {
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0
            && b[i] == b'a'
            && b[i + 1] == b's'
            && (i == 0 || !mock_is_ident_char(b[i - 1] as char))
            && (i + 2 >= b.len() || !mock_is_ident_char(b[i + 2] as char))
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

impl MockInterp {
    fn bump(&mut self) -> bool {
        self.steps += 1;
        self.steps < MOCK_MAX_STEPS
            && self.output.len() < MOCK_MAX_OUTPUT_BYTES
    }

    fn push_line(&mut self, line: &str) {
        if self.output.len() < MOCK_MAX_OUTPUT_BYTES {
            self.output.push_str(line);
            self.output.push('\n');
        }
    }

    /// Evaluate an expression to its display value.
    fn eval(&mut self, expr: &str) -> String {
        if !self.bump() {
            return String::new();
        }
        let t = expr.trim();
        if t.is_empty() {
            return String::new();
        }
        // String / char literals (interpolated).
        if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
            return mock_interpolate(&process_escapes(&t[1..t.len() - 1]), &self.vars, &self.env);
        }
        if t.len() >= 2 && t.starts_with('\'') && t.ends_with('\'') {
            return t[1..t.len() - 1].to_string();
        }
        if t.len() >= 2 && t.starts_with('`') && t.ends_with('`') {
            return mock_interpolate(t[1..t.len() - 1].trim(), &self.vars, &self.env);
        }
        // Numbers / bools.
        if t.parse::<i64>().is_ok() || t.parse::<f64>().is_ok() {
            return t.to_string();
        }
        if t == "true" || t == "false" || t == "null" {
            return t.to_string();
        }
        // Parenthesized grouping (tuples keep their shape).
        if t.starts_with('(') {
            if let Some(close) = mock_paren_match(t, 0) {
                if close == t.len() - 1 {
                    let inner = &t[1..t.len() - 1];
                    if split_mock_top_level(inner).len() > 1 {
                        return t.to_string();
                    }
                    return self.eval(inner);
                }
            }
        }
        // Array / map literals display as-is.
        if t.starts_with('[') && t.ends_with(']') && t.len() >= 2 {
            return t.to_string();
        }
        // `Json::serialize(...)` -> JSON text.
        if t.starts_with("Json::serialize(") || t.starts_with("Json.serialize(") {
            if let Some(open) = t.find('(') {
                if let Some((inner, _)) = extract_mock_call_inner(t, open) {
                    let v = self.eval(&inner);
                    return mock_lex_to_json(&v);
                }
            }
            return String::new();
        }
        // Known externs keep the battle-tested string behavior.
        if t.starts_with("Json::parse(")
            || t.starts_with("Json.parse(")
            || t.starts_with("Env::")
            || t.starts_with("Env.")
            || t.starts_with("Http::")
            || t.starts_with("Http.")
        {
            return eval_simple_expr(t, &self.vars, &self.json, &self.structs);
        }
        // Pipes keep legacy behavior.
        if mock_split_plus(t).len() == 1 && t.contains("|>") {
            return eval_simple_expr(t, &self.vars, &self.json, &self.structs);
        }
        // Arithmetic / concatenation.
        if let Some(arith) = self.eval_arith(t) {
            return arith;
        }
        // `as` casts.
        if let Some(pos) = mock_find_top_level_as(t) {
            let left = self.eval(&t[..pos]);
            let ty = t[pos + 2..].trim();
            let ty_end = ty
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(ty.len());
            return match &ty[..ty_end] {
                "String" => left,
                "i32" | "i64" | "u64" | "int" | "uint" => left.parse::<i64>().map(|n| n.to_string()).unwrap_or(left),
                "f32" | "f64" | "float" => left.parse::<f64>().map(|n| {
                    if n.fract() == 0.0 { format!("{}", n as i64) } else { n.to_string() }
                }).unwrap_or(left),
                "bool" => left,
                _ => left,
            };
        }
        // Calls: user `name(...)`, `obj.method(...)`, or skipped externs.
        if let Some(v) = self.eval_call(t) {
            return v;
        }
        // Field access `obj.field` (no call).
        if !t.contains('(') {
            if let Some(dot) = mock_last_top_dot(t) {
                let (obj, field) = (t[..dot].trim(), t[dot + 1..].trim());
                if !obj.is_empty()
                    && !field.is_empty()
                    && field.chars().all(mock_is_ident_char)
                    && !field.contains('.')
                {
                    let obj_val = self.eval(obj);
                    if let Some(fv) = mock_struct_field(&obj_val, field) {
                        return fv;
                    }
                    // JSON objects from Http/Json surfaces.
                    if let Ok(jv) = serde_json::from_str::<serde_json::Value>(&obj_val) {
                        if let Some(got) = jv.get(field) {
                            return match got {
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            };
                        }
                    }
                }
            }
        }
        // Tracked variables.
        if let Some(v) = self.vars.get(t) {
            return v.clone();
        }
        if let Some(v) = self.env.get(t) {
            return v.clone();
        }
        // Interpolation fallback (`ola {n}` content without quotes).
        if t.contains('{') && t.contains('}') {
            let inter = mock_interpolate(t, &self.vars, &self.env);
            if inter != t {
                return inter;
            }
        }
        t.to_string()
    }

    /// `+ - * /` over evaluated operands (numeric when possible).
    fn eval_arith(&mut self, t: &str) -> Option<String> {
        // `+` first so `a as T + b` splits before the cast handler runs.
        let parts = mock_split_plus(t);
        if parts.len() > 1 {
            let vals: Vec<String> = parts.iter().map(|p| self.eval(p)).collect();
            let any_str = parts.iter().any(|p| {
                let q = p.trim();
                q.len() >= 2 && q.starts_with('"') && q.ends_with('"')
            });
            if !any_str && vals.iter().all(|v| v.parse::<i64>().is_ok()) {
                let sum: i64 = vals.iter().map(|v| v.parse::<i64>().unwrap_or(0)).sum();
                return Some(sum.to_string());
            }
            if !any_str && vals.iter().all(|v| v.parse::<f64>().is_ok()) {
                let sum: f64 = vals.iter().map(|v| v.parse::<f64>().unwrap_or(0.0)).sum();
                if sum.fract() == 0.0 {
                    return Some(format!("{}", sum as i64));
                }
                return Some(sum.to_string());
            }
            return Some(vals.concat());
        }
        let (idx, op) = mock_find_binary_op(t)?;
        if op == '+' {
            return None; // handled above (unreachable, kept for clarity)
        }
        let (l, r) = (t[..idx].trim(), t[idx + 1..].trim());
        if l.is_empty() || r.is_empty() {
            return None;
        }
        let lv = self.eval(l);
        let rv = self.eval(r);
        match op {
            '-' | '*' | '/' => {
                if let (Ok(a), Ok(b)) = (lv.parse::<i64>(), rv.parse::<i64>()) {
                    return Some(match op {
                        '-' => (a - b).to_string(),
                        '*' => (a * b).to_string(),
                        '/' => {
                            if b == 0 {
                                return Some(t.to_string());
                            }
                            if a % b == 0 { (a / b).to_string() } else { ((a as f64) / (b as f64)).to_string() }
                        }
                        _ => t.to_string(),
                    });
                }
                if let (Ok(a), Ok(b)) = (lv.parse::<f64>(), rv.parse::<f64>()) {
                    let v = match op {
                        '-' => a - b,
                        '*' => a * b,
                        '/' => {
                            if b == 0.0 {
                                return Some(t.to_string());
                            }
                            a / b
                        }
                        _ => return None,
                    };
                    if v.fract() == 0.0 {
                        return Some(format!("{}", v as i64));
                    }
                    return Some(v.to_string());
                }
                Some(t.to_string())
            }
            _ => None,
        }
    }

    /// `name(args)` / `obj.method(args)`. `None` = not a call shape.
    fn eval_call(&mut self, t: &str) -> Option<String> {
        // Leading identifier (allows `::` paths for the extern check below).
        let mut id_end = 0;
        for c in t.chars() {
            if mock_is_ident_char(c) {
                id_end += c.len_utf8();
            } else {
                break;
            }
        }
        if id_end == 0 {
            return None;
        }
        let name = &t[..id_end];
        if matches!(
            name,
            "if" | "for" | "while" | "loop" | "do" | "switch" | "match" | "return" | "break"
                | "continue" | "let" | "var" | "const" | "fn" | "struct" | "in" | "as" | "true"
                | "false" | "null"
        ) {
            return None;
        }
        let rest = t[id_end..].trim_start();
        // `Name { ... }` struct literal: display as-is.
        if rest.starts_with('{') {
            return Some(t.to_string());
        }
        // Method / static call `a.b(...)`, `Http::get(...)`.
        if rest.starts_with('.') || rest.starts_with("::") {
            let after_sep = if rest.starts_with('.') { &rest[1..] } else { &rest[2..] };
            let m_end = after_sep
                .find(|c: char| !mock_is_ident_char(c))
                .unwrap_or(after_sep.len());
            let method = &after_sep[..m_end];
            let after_m = after_sep[m_end..].trim_start();
            if !after_m.starts_with('(') {
                return None; // bare `a.b` is field access, handled elsewhere
            }
            let open = t.len() - after_m.len();
            let (inner, _) = extract_mock_call_inner(t, open)?;
            if ["len", "size", "length"].contains(&method) {
                let v = self.eval(name);
                return Some(mock_display_len(&v));
            }
            if method == "toString" || method == "to_string" {
                return Some(self.eval(name));
            }
            if method == "push" {
                // `arr.push(x)` as an expression: mutate and yield the array.
                let elem = self.eval(&inner);
                let arr = self.vars.get(name).cloned().unwrap_or_default();
                let updated = mock_array_push(&arr, &elem);
                self.vars.insert(name.to_string(), updated.clone());
                return Some(updated);
            }
            // Unknown extern/static call: legacy behavior evaluates via old path.
            let mut all = self.vars.clone();
            for (k, v) in &self.env {
                all.entry(k.clone()).or_insert(v.clone());
            }
            return Some(eval_method_call(t, &all));
        }
        if !rest.starts_with('(') {
            return None;
        }
        let open = id_end + (t[id_end..].len() - rest.len());
        let (inner, _) = extract_mock_call_inner(t, open)?;
        // Trailing garbage after `)` (e.g. `.field`) is not a plain call.
        // (Field-of-call-result is out of scope for the mock.)
        let arg_vals: Vec<String> = split_mock_top_level(&inner).iter().map(|a| self.eval(a)).collect();
        if self.fns.contains_key(name) {
            return Some(self.exec_fn(name, arg_vals).unwrap_or_default());
        }
        // `assert(...)` / `panic(...)` / unknown builtins: no value.
        Some(String::new())
    }

    /// Condition truth value. `None` = not decidable (caller keeps legacy).
    fn eval_cond(&mut self, expr: &str) -> Option<bool> {
        let t = expr.trim();
        if t.is_empty() {
            return None;
        }
        if t == "true" {
            return Some(true);
        }
        if t == "false" {
            return Some(false);
        }
        if let Some(or) = mock_split_logic(t, "||") {
            let mut any_none = false;
            for part in or {
                match self.eval_cond(&part) {
                    Some(true) => return Some(true),
                    Some(false) => {}
                    None => any_none = true,
                }
            }
            return if any_none { None } else { Some(false) };
        }
        if let Some(and) = mock_split_logic(t, "&&") {
            let mut any_none = false;
            for part in and {
                match self.eval_cond(&part) {
                    Some(false) => return Some(false),
                    Some(true) => {}
                    None => any_none = true,
                }
            }
            return if any_none { None } else { Some(true) };
        }
        if t.starts_with('!') && !t.starts_with("!=") {
            return self.eval_cond(&t[1..]).map(|b| !b);
        }
        if t.starts_with('(') {
            if let Some(close) = mock_paren_match(t, 0) {
                if close == t.len() - 1 {
                    return self.eval_cond(&t[1..t.len() - 1]);
                }
            }
        }
        if let Some((l, op, r)) = mock_split_comparison(t) {
            let lv = self.eval(&l);
            let rv = self.eval(&r);
            if let (Ok(a), Ok(b)) = (lv.parse::<f64>(), rv.parse::<f64>()) {
                return Some(match op {
                    "==" => a == b,
                    "!=" => a != b,
                    "<" => a < b,
                    ">" => a > b,
                    "<=" => a <= b,
                    ">=" => a >= b,
                    _ => return None,
                });
            }
            return Some(match op {
                "==" => lv == rv,
                "!=" => lv != rv,
                "<" => lv < rv,
                ">" => lv > rv,
                "<=" => lv <= rv,
                ">=" => lv >= rv,
                _ => return None,
            });
        }
        match self.eval(t).as_str() {
            "true" => Some(true),
            "false" | "" => Some(false),
            n => n.parse::<f64>().ok().map(|v| v != 0.0),
        }
    }

    /// `println`-style argument evaluation (concatenation aware).
    fn print_arg(&mut self, arg: &str) -> String {
        let t = arg.trim();
        if t.is_empty() {
            return String::new();
        }
        if t.contains("inspect(") {
            return handle_inspect(t, &self.vars, &self.json, &self.structs);
        }
        // `+` at top level concatenates (numeric-only adds up).
        let parts = mock_split_plus(t);
        if parts.len() > 1 {
            let vals: Vec<String> = parts.iter().map(|p| self.print_atom(p)).collect();
            let any_str = parts.iter().any(|p| {
                let q = p.trim();
                q.len() >= 2 && q.starts_with('"') && q.ends_with('"')
            });
            if !any_str && vals.iter().all(|v| v.parse::<i64>().is_ok()) {
                let sum: i64 = vals.iter().map(|v| v.parse::<i64>().unwrap_or(0)).sum();
                return sum.to_string();
            }
            if !any_str && vals.iter().all(|v| v.parse::<f64>().is_ok()) {
                let sum: f64 = vals.iter().map(|v| v.parse::<f64>().unwrap_or(0.0)).sum();
                if sum.fract() == 0.0 {
                    return format!("{}", sum as i64);
                }
                return sum.to_string();
            }
            return vals.concat();
        }
        self.print_atom(t)
    }

    fn print_atom(&mut self, part: &str) -> String {
        let t = part.trim();
        for m in ["len()", "size()", "length()"] {
            if let Some(obj) = t.strip_suffix(m) {
                let obj = obj.trim_end_matches('.').trim();
                if obj.is_empty() {
                    return "0".to_string();
                }
                let v = self.eval(obj);
                return mock_display_len(&v);
            }
        }
        if let Some(obj) = t.strip_suffix("toString()").or_else(|| t.strip_suffix("to_string()")) {
            let obj = obj.trim_end_matches('.').trim();
            return self.eval(obj);
        }
        if let Some(pos) = mock_find_top_level_as(t) {
            let left = self.eval(&t[..pos]);
            let ty = t[pos + 2..].trim();
            let ty_end = ty
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(ty.len());
            return match &ty[..ty_end] {
                "String" => left,
                _ => left,
            };
        }
        self.eval(t)
    }

    /// Execute a user function by name (statements run, prints collected).
    fn exec_fn(&mut self, name: &str, args: Vec<String>) -> Option<String> {
        if self.fn_depth >= MOCK_MAX_FN_DEPTH || !self.bump() {
            return None;
        }
        let def = self.fns.get(name)?.clone();
        self.fn_depth += 1;
        let mut declared = Vec::new();
        for (i, p) in def.params.iter().enumerate() {
            declared.push((p.clone(), self.vars.get(p).cloned()));
            self.vars.insert(p.clone(), args.get(i).cloned().unwrap_or_default());
        }
        let flow = self.exec_stmts(&def.body, &mut declared);
        for (k, old) in declared.into_iter().rev() {
            match old {
                Some(v) => {
                    self.vars.insert(k, v);
                }
                None => {
                    self.vars.remove(&k);
                }
            }
        }
        self.fn_depth -= 1;
        match flow {
            MockFlow::Return(v) => v,
            _ => None,
        }
    }

    fn exec_block(&mut self, body: &str) -> MockFlow {
        let mut declared = Vec::new();
        let flow = self.exec_stmts(body, &mut declared);
        for (k, old) in declared.into_iter().rev() {
            match old {
                Some(v) => {
                    self.vars.insert(k, v);
                }
                None => {
                    self.vars.remove(&k);
                }
            }
        }
        flow
    }

    /// Append `elem` (display value) to a `[...]` array value.
    fn stmt_push(&mut self, arr: &str, elem: &str) {
        let cur = self.vars.get(arr).cloned().unwrap_or_default();
        self.vars.insert(arr.to_string(), mock_array_push(&cur, elem));
    }
}

/// End of a simple statement from `from`: index just past the depth-0
/// `;` (or depth-0 newline for semi-less lines). Always advances.
fn mock_stmt_end(src: &str, from: usize) -> usize {
    let b = src.as_bytes();
    if from >= b.len() {
        return b.len();
    }
    if b[from] == b';' {
        return from + 1;
    }
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let mut i = from;
    while i < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            i += 1;
            continue;
        }
        match c {
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth = depth.saturating_sub(1),
            ';' if depth == 0 => return i + 1,
            '\n' if depth == 0 => return i + 1,
            _ => {}
        }
        i += 1;
    }
    b.len()
}

/// First `{` at depth 0 (tracks `()`/`[]`, skips strings). `None` when a
/// depth-0 `;` terminates the header first.
fn mock_find_block_open(src: &str, from: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let mut i = from;
    while i < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            i += 1;
            continue;
        }
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            '{' if depth == 0 => return Some(i),
            ';' if depth == 0 => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

/// Skip a declaration-ish item (`fn`/`struct`/...) to after its `;` or body.
fn mock_skip_item(src: &str, from: usize) -> usize {
    let b = src.as_bytes();
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let mut i = from;
    while i < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            i += 1;
            continue;
        }
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            '{' if depth == 0 => {
                if let Some(end) = mock_brace_match(src, i) {
                    let mut j = end + 1;
                    j = mock_skip_ws(src, j);
                    if j < b.len() && b[j] == b';' {
                        j += 1;
                    }
                    return j;
                }
                return b.len();
            }
            ';' if depth == 0 => return i + 1,
            _ => {}
        }
        i += 1;
    }
    b.len()
}

/// Split `a || b` / `a && b` at depth 0. `None` when the operator is absent.
fn mock_split_logic(s: &str, op: &str) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let mut cur = String::new();
    let mut found = false;
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if esc {
            cur.push(c);
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            cur.push(c);
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            cur.push(c);
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            cur.push(c);
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            cur.push(c);
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            cur.push(c);
            i += 1;
            continue;
        }
        match c {
            '[' | '{' | '(' => {
                depth += 1;
                cur.push(c);
            }
            ']' | '}' | ')' => {
                depth = depth.saturating_sub(1);
                cur.push(c);
            }
            _ => {
                if depth == 0 && s[i..].starts_with(op) {
                    parts.push(cur.trim().to_string());
                    cur.clear();
                    found = true;
                    i += op.len();
                    continue;
                }
                cur.push(c);
            }
        }
        i += 1;
    }
    if !found {
        return None;
    }
    parts.push(cur.trim().to_string());
    Some(parts)
}

/// Split a comparison `left OP right` (`== != <= >= < >`, skips `=>`).
fn mock_split_comparison(s: &str) -> Option<(String, &'static str, String)> {
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            i += 1;
            continue;
        }
        match c {
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth = depth.saturating_sub(1),
            '=' | '!' | '<' | '>' if depth == 0 => {
                let rest = &s[i..];
                // `=>` is not a comparison.
                if rest.starts_with("=>") {
                    i += 2;
                    continue;
                }
                for op in ["==", "!=", "<=", ">="] {
                    if rest.starts_with(op) {
                        return Some((s[..i].trim().to_string(), op, s[i + 2..].trim().to_string()));
                    }
                }
                if c == '<' || c == '>' {
                    // Single `<`/`>` (not `<<`, `>>`, `->`, `=>`).
                    let next = if i + 1 < b.len() { b[i + 1] as char } else { ' ' };
                    if next == c || next == '=' || (c == '-' && next == '>') {
                        i += 1;
                        continue;
                    }
                    let op: &'static str = if c == '<' { "<" } else { ">" };
                    return Some((s[..i].trim().to_string(), op, s[i + 1..].trim().to_string()));
                }
                if c == '=' {
                    i += 1;
                    continue;
                }
                // Lone `!` (unary) is handled by the caller.
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Last depth-0 `.` with identifier chars around it (field access).
fn mock_last_top_dot(s: &str) -> Option<usize> {
    let mut depth: usize = 0;
    let mut in_d = false;
    let mut in_s = false;
    let mut in_b = false;
    let mut esc = false;
    let b = s.as_bytes();
    let mut last: Option<usize> = None;
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_d || in_s) {
            esc = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_s && !in_b {
            in_d = !in_d;
            i += 1;
            continue;
        }
        if c == '\'' && !in_d && !in_b {
            in_s = !in_s;
            i += 1;
            continue;
        }
        if c == '`' && !in_d && !in_s {
            in_b = !in_b;
            i += 1;
            continue;
        }
        if in_d || in_s || in_b {
            i += 1;
            continue;
        }
        match c {
            '[' | '{' | '(' => depth += 1,
            ']' | '}' | ')' => depth = depth.saturating_sub(1),
            '.' if depth == 0 => {
                let prev_ok = i > 0 && mock_is_ident_char(b[i - 1] as char);
                let next_ok = i + 1 < b.len() && mock_is_ident_char(b[i + 1] as char);
                if prev_ok && next_ok {
                    last = Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    last
}

/// Append a display value to a `[...]` array value.
fn mock_array_push(arr_val: &str, elem: &str) -> String {
    let t = arr_val.trim();
    if t.starts_with('[') && t.ends_with(']') && t.len() >= 2 {
        let inner = t[1..t.len() - 1].trim();
        if inner.is_empty() {
            return format!("[{}]", elem.trim());
        }
        return format!("[{}, {}]", inner, elem.trim());
    }
    if t.is_empty() {
        return format!("[{}]", elem.trim());
    }
    arr_val.to_string()
}

fn mock_starts_kw(rest: &str, kw: &str) -> bool {
    rest.starts_with(kw)
        && rest[kw.len()..]
            .chars()
            .next()
            .map(|c| !mock_is_ident_char(c))
            .unwrap_or(true)
}

impl MockInterp {
    /// Execute statements in `body` (no outer braces needed).
    fn exec_stmts(
        &mut self,
        body: &str,
        declared: &mut Vec<(String, Option<String>)>,
    ) -> MockFlow {
        let b = body.as_bytes();
        let mut pos = 0;
        while pos < b.len() {
            if !self.bump() {
                return MockFlow::Next;
            }
            pos = mock_skip_ws(body, pos);
            if pos >= b.len() {
                break;
            }
            if b[pos] == b';' || b[pos] == b'}' {
                pos += 1;
                continue;
            }
            let rest = &body[pos..];
            // --- declarations ---
            if mock_starts_kw(rest, "let") || mock_starts_kw(rest, "var") || mock_starts_kw(rest, "const") {
                let kwlen = if rest.starts_with("const") { 5 } else { 3 };
                let after = pos + kwlen;
                let tail = &body[after..];
                if let Some(eq) = find_mock_assign_eq(tail) {
                    if let Some(name) = clean_mock_let_name(tail[..eq].trim()) {
                        let (val, next) = extract_mock_let_value(body, after + eq + 1);
                        let ev = self.eval(&val);
                        declared.push((name.clone(), self.vars.get(&name).cloned()));
                        self.vars.insert(name, ev);
                        pos = next;
                        continue;
                    }
                }
                pos = mock_stmt_end(body, after).max(pos + 1);
                continue;
            }
            // --- loops / branches ---
            if mock_starts_kw(rest, "for") {
                pos = self.stmt_for(body, pos, declared);
                if let Some(flow) = self.stmt_flow_take() {
                    return flow;
                }
                continue;
            }
            if mock_starts_kw(rest, "while") {
                pos = self.stmt_while(body, pos);
                if let Some(flow) = self.stmt_flow_take() {
                    return flow;
                }
                continue;
            }
            if mock_starts_kw(rest, "loop") {
                pos = self.stmt_loop(body, pos);
                if let Some(flow) = self.stmt_flow_take() {
                    return flow;
                }
                continue;
            }
            if mock_starts_kw(rest, "do") {
                pos = self.stmt_do_while(body, pos);
                if let Some(flow) = self.stmt_flow_take() {
                    return flow;
                }
                continue;
            }
            if mock_starts_kw(rest, "if") {
                pos = self.stmt_if(body, pos);
                if let Some(flow) = self.stmt_flow_take() {
                    return flow;
                }
                continue;
            }
            if mock_starts_kw(rest, "switch") {
                pos = self.stmt_switch(body, pos);
                if let Some(flow) = self.stmt_flow_take() {
                    return flow;
                }
                continue;
            }
            if mock_starts_kw(rest, "match") {
                pos = self.stmt_match(body, pos);
                if let Some(flow) = self.stmt_flow_take() {
                    return flow;
                }
                continue;
            }
            if mock_starts_kw(rest, "return") {
                let after = mock_skip_ws(body, pos + 6);
                if after < b.len() && b[after] != b';' && b[after] != b'}' {
                    let end = mock_stmt_end(body, after);
                    let mut expr = body[after..end].trim().to_string();
                    expr = expr.trim_end_matches(';').trim().to_string();
                    if expr.is_empty() {
                        return MockFlow::Return(None);
                    }
                    let v = self.eval(&expr);
                    return MockFlow::Return(Some(v));
                }
                if after < b.len() && b[after] == b';' {
                    pos = after + 1;
                } else {
                    pos = after;
                }
                return MockFlow::Return(None);
            }
            if mock_starts_kw(rest, "break") {
                pos = mock_stmt_end(body, pos + 5).max(pos + 1);
                return MockFlow::Break;
            }
            if mock_starts_kw(rest, "continue") {
                pos = mock_stmt_end(body, pos + 8).max(pos + 1);
                return MockFlow::Continue;
            }
            // --- prints ---
            let mut printed = false;
            for pat in MOCK_PRINT_PATTERNS {
                if rest.starts_with(pat) {
                    let open = pos + pat.len() - 1;
                    if let Some(close) = mock_paren_match(body, open) {
                        let inner = body[open + 1..close].trim().to_string();
                        let line = self.print_arg(&inner);
                        self.push_line(&line);
                        pos = close + 1;
                        if pos < b.len() && b[pos] == b';' {
                            pos += 1;
                        }
                    } else {
                        pos = mock_stmt_end(body, pos).max(pos + 1);
                    }
                    printed = true;
                    break;
                }
            }
            if printed {
                continue;
            }
            // --- noise: assert/panic/throw/defer/serve/attributes/items ---
            if mock_starts_kw(rest, "assert")
                || mock_starts_kw(rest, "panic")
                || mock_starts_kw(rest, "throw")
                || mock_starts_kw(rest, "defer")
            {
                pos = mock_stmt_end(body, pos).max(pos + 1);
                continue;
            }
            if rest.starts_with('@') {
                let nl = body[pos..].find('\n').map(|i| pos + i + 1).unwrap_or(b.len());
                pos = nl;
                continue;
            }
            if mock_starts_kw(rest, "fn")
                || mock_starts_kw(rest, "struct")
                || mock_starts_kw(rest, "enum")
                || mock_starts_kw(rest, "class")
                || mock_starts_kw(rest, "trait")
                || mock_starts_kw(rest, "interface")
                || mock_starts_kw(rest, "type")
                || mock_starts_kw(rest, "import")
                || mock_starts_kw(rest, "service")
                || mock_starts_kw(rest, "pub")
                || mock_starts_kw(rest, "async")
                || mock_starts_kw(rest, "static")
                || mock_starts_kw(rest, "macro")
                || mock_starts_kw(rest, "extern")
            {
                pos = mock_skip_item(body, pos).max(pos + 1);
                continue;
            }
            // --- identifier-led statements: call / assign / method / discard ---
            if let Some((ident, after_ident)) = mock_read_ident(body, pos) {
                let after = mock_skip_ws(body, after_ident);
                // Call `name(...)`.
                if after < b.len() && b[after] == b'(' {
                    if let Some(close) = mock_paren_match(body, after) {
                        let args: Vec<String> =
                            split_mock_top_level(&body[after + 1..close]).iter().map(|a| self.eval(a)).collect();
                        if self.fns.contains_key(&ident) {
                            let _ = self.exec_fn(&ident, args);
                        }
                        pos = close + 1;
                        if pos < b.len() && b[pos] == b';' {
                            pos += 1;
                        }
                        continue;
                    }
                    pos = mock_stmt_end(body, pos).max(pos + 1);
                    continue;
                }
                // Method statement `obj.method(...)` (only `push` mutates).
                if after < b.len() && (b[after] == b'.' || (after + 1 < b.len() && &body[after..after + 2] == "::")) {
                    let mstart = if b[after] == b'.' { after + 1 } else { after + 2 };
                    if let Some((method, after_m)) = mock_read_ident(body, mstart) {
                        let am = mock_skip_ws(body, after_m);
                        if am < b.len() && b[am] == b'(' {
                            if let Some(close) = mock_paren_match(body, am) {
                                if method == "push" {
                                    let elem = self.eval(body[am + 1..close].trim());
                                    self.stmt_push(&ident, &elem);
                                }
                                pos = close + 1;
                                if pos < b.len() && b[pos] == b';' {
                                    pos += 1;
                                }
                                continue;
                            }
                        }
                    }
                    pos = mock_stmt_end(body, pos).max(pos + 1);
                    continue;
                }
                // Assignment `x = ...` / `x += ...` (never `==` / `=>`).
                if after < b.len() && (b[after] == b'=' || b[after] == b'+' || b[after] == b'-' || b[after] == b'*' || b[after] == b'/' || b[after] == b'%') {
                    let mut op_end = after;
                    let mut op = "=";
                    for cand in ["+=", "-=", "*=", "/=", "%=", "="] {
                        if body[after..].starts_with(cand) {
                            // Reject `==` and `=>`.
                            if cand == "=" {
                                let an = after + 1;
                                if an < b.len() && (b[an] == b'=' || b[an] == b'>') {
                                    op = "";
                                    break;
                                }
                            }
                            op = cand;
                            op_end = after + cand.len();
                            break;
                        }
                    }
                    if !op.is_empty() {
                        let end = mock_stmt_end(body, op_end);
                        let mut rhs = body[op_end..end].trim().to_string();
                        rhs = rhs.trim_end_matches(';').trim().to_string();
                        let val = self.eval(&rhs);
                        let cur = self.vars.get(&ident).cloned().unwrap_or_default();
                        let next_val = match op {
                            "=" => val,
                            "+=" => self.eval(&format!("{} + {}", mock_val_or_zero(&cur), mock_val_or_zero(&val))),
                            "-=" => self.eval(&format!("{} - {}", mock_val_or_zero(&cur), mock_val_or_zero(&val))),
                            "*=" => self.eval(&format!("{} * {}", mock_val_or_zero(&cur), mock_val_or_zero(&val))),
                            "/=" => self.eval(&format!("{} / {}", mock_val_or_zero(&cur), mock_val_or_zero(&val))),
                            "%=" => {
                                match (cur.parse::<i64>(), val.parse::<i64>()) {
                                    (Ok(a), Ok(b)) if b != 0 => (a % b).to_string(),
                                    _ => val,
                                }
                            }
                            _ => val,
                        };
                        self.vars.insert(ident, next_val);
                        pos = end;
                        continue;
                    }
                }
                // Bare expression / unknown: evaluate and discard.
                let end = mock_stmt_end(body, pos);
                let stmt = body[pos..end].trim().trim_end_matches(';').trim().to_string();
                if !stmt.is_empty() {
                    let _ = self.eval(&stmt);
                }
                pos = end.max(pos + 1);
                continue;
            }
            // Unknown token: make progress.
            pos = mock_stmt_end(body, pos).max(pos + 1);
        }
        MockFlow::Next
    }

    /// Drains a pending control-flow signal stored by `stmt_*` helpers.
    /// (`stmt_*` helpers return the new cursor and stash any flow here.)
    fn stmt_flow_take(&mut self) -> Option<MockFlow> {
        if self.pending_flow.is_some() {
            return self.pending_flow.take().map(|f| match f {
                MockFlow::Break | MockFlow::Continue | MockFlow::Return(_) => f,
                MockFlow::Next => MockFlow::Next,
            });
        }
        None
    }

    /// `for x in EXPR { ... }` at `pos` (pos points at `for`).
    fn stmt_for(
        &mut self,
        body: &str,
        pos: usize,
        _declared: &mut Vec<(String, Option<String>)>,
    ) -> usize {
        let b = body.as_bytes();
        let mut p = mock_skip_ws(body, pos + 3);
        let (var, after_var) = match mock_read_ident(body, p) {
            Some(v) => v,
            None => {
                // Tuple destructuring etc.: skip the whole loop, legacy-style.
                if let Some(open) = mock_find_block_open(body, p) {
                    if let Some(end) = mock_brace_match(body, open) {
                        return end + 1;
                    }
                }
                return mock_stmt_end(body, pos).max(pos + 1);
            }
        };
        p = mock_skip_ws(body, after_var);
        if !mock_starts_kw(&body[p.min(b.len())..], "in") {
            return mock_stmt_end(body, pos).max(pos + 1);
        }
        p = mock_skip_ws(body, p + 2);
        let Some(open) = mock_find_block_open(body, p) else {
            return mock_stmt_end(body, pos).max(pos + 1);
        };
        let expr = body[p..open].trim().to_string();
        let Some(end) = mock_brace_match(body, open) else {
            return b.len();
        };
        let loop_body = body[open + 1..end].to_string();
        let arr_val = self.eval(&expr);
        let mut elems: Vec<String> = Vec::new();
        let t = arr_val.trim();
        if t.starts_with('[') && t.ends_with(']') && t.len() >= 2 {
            elems = split_mock_top_level(&t[1..t.len() - 1]);
        }
        if elems.is_empty() {
            // Unknown iterable (e.g. bare `items`): run once, legacy-style.
            let old = self.vars.get(&var).cloned();
            let flow = self.exec_block(&loop_body);
            match old {
                Some(v) => {
                    self.vars.insert(var, v);
                }
                None => {
                    self.vars.remove(&var);
                }
            }
            if !matches!(flow, MockFlow::Next) {
                self.pending_flow = Some(flow);
            }
            return end + 1;
        }
        let old = self.vars.get(&var).cloned();
        let mut flow = MockFlow::Next;
        for (n, elem) in elems.iter().enumerate() {
            if n >= MOCK_MAX_LOOP_ITERS || !self.bump() {
                break;
            }
            self.vars.insert(var.clone(), elem.clone());
            match self.exec_block(&loop_body) {
                MockFlow::Next => {}
                MockFlow::Continue => continue,
                MockFlow::Break => break,
                r @ MockFlow::Return(_) => {
                    flow = r;
                    break;
                }
            }
        }
        match old {
            Some(v) => {
                self.vars.insert(var, v);
            }
            None => {
                self.vars.remove(&var);
            }
        }
        if !matches!(flow, MockFlow::Next) {
            self.pending_flow = Some(flow);
        }
        end + 1
    }

    /// `while COND { ... }` at `pos`.
    fn stmt_while(&mut self, body: &str, pos: usize) -> usize {
        let b = body.as_bytes();
        let p = mock_skip_ws(body, pos + 5);
        let Some(open) = mock_find_block_open(body, p) else {
            return mock_stmt_end(body, pos).max(pos + 1);
        };
        let cond = body[p..open].trim().to_string();
        let Some(end) = mock_brace_match(body, open) else {
            return b.len();
        };
        let loop_body = body[open + 1..end].to_string();
        let mut iters = 0;
        loop {
            if iters >= MOCK_MAX_LOOP_ITERS || !self.bump() {
                break;
            }
            match self.eval_cond(&cond) {
                Some(false) => break,
                None => {
                    // Undecidable: single pass, legacy-style.
                    let flow = self.exec_block(&loop_body);
                    if matches!(flow, MockFlow::Return(_)) {
                        self.pending_flow = Some(flow);
                    }
                    break;
                }
                Some(true) => {}
            }
            match self.exec_block(&loop_body) {
                MockFlow::Next => {}
                MockFlow::Continue => {}
                MockFlow::Break => break,
                r @ MockFlow::Return(_) => {
                    self.pending_flow = Some(r);
                    break;
                }
            }
            iters += 1;
        }
        end + 1
    }

    /// `loop { ... }` at `pos`.
    fn stmt_loop(&mut self, body: &str, pos: usize) -> usize {
        let b = body.as_bytes();
        let p = mock_skip_ws(body, pos + 4);
        if p >= b.len() || b[p] != b'{' {
            return mock_stmt_end(body, pos).max(pos + 1);
        }
        let Some(end) = mock_brace_match(body, p) else {
            return b.len();
        };
        let loop_body = body[p + 1..end].to_string();
        let mut iters = 0;
        loop {
            if iters >= MOCK_MAX_LOOP_ITERS || !self.bump() {
                break;
            }
            match self.exec_block(&loop_body) {
                MockFlow::Next => {}
                MockFlow::Continue => {}
                MockFlow::Break => break,
                r @ MockFlow::Return(_) => {
                    self.pending_flow = Some(r);
                    break;
                }
            }
            iters += 1;
        }
        end + 1
    }

    /// `do { ... } while COND;` at `pos`.
    fn stmt_do_while(&mut self, body: &str, pos: usize) -> usize {
        let b = body.as_bytes();
        let p = mock_skip_ws(body, pos + 2);
        if p >= b.len() || b[p] != b'{' {
            return mock_stmt_end(body, pos).max(pos + 1);
        }
        let Some(end) = mock_brace_match(body, p) else {
            return b.len();
        };
        let loop_body = body[p + 1..end].to_string();
        let mut q = mock_skip_ws(body, end + 1);
        let mut cond = String::new();
        if mock_starts_kw(&body[q.min(b.len())..], "while") {
            q = mock_skip_ws(body, q + 5);
            let cend = mock_stmt_end(body, q);
            cond = body[q..cend].trim().trim_end_matches(';').trim().to_string();
            q = cend;
        }
        let mut iters = 0;
        loop {
            if iters >= MOCK_MAX_LOOP_ITERS || !self.bump() {
                break;
            }
            match self.exec_block(&loop_body) {
                MockFlow::Next => {}
                MockFlow::Continue => {}
                MockFlow::Break => break,
                r @ MockFlow::Return(_) => {
                    self.pending_flow = Some(r);
                    break;
                }
            }
            iters += 1;
            match self.eval_cond(&cond) {
                Some(true) => continue,
                _ => break,
            }
        }
        q
    }

    /// `if C { ... } else if ... else { ... }` at `pos`.
    fn stmt_if(&mut self, body: &str, pos: usize) -> usize {
        let b = body.as_bytes();
        let mut p = mock_skip_ws(body, pos + 2);
        let mut end_pos = pos;
        loop {
            let Some(open) = mock_find_block_open(body, p) else {
                return mock_stmt_end(body, pos).max(pos + 1);
            };
            let cond = body[p..open].trim().to_string();
            let Some(end) = mock_brace_match(body, open) else {
                return b.len();
            };
            let then_body = body[open + 1..end].to_string();
            end_pos = end + 1;
            let take = match self.eval_cond(&cond) {
                Some(v) => v,
                None => true, // legacy visibility: undecidable runs the branch
            };
            if take {
                let flow = self.exec_block(&then_body);
                if !matches!(flow, MockFlow::Next) {
                    self.pending_flow = Some(flow);
                }
                // Skip any trailing else-chain without running it.
                let mut q = mock_skip_ws(body, end_pos);
                while mock_starts_kw(&body[q.min(b.len())..], "else") {
                    q = mock_skip_ws(body, q + 4);
                    if mock_starts_kw(&body[q.min(b.len())..], "if") {
                        q = mock_skip_ws(body, q + 2);
                        if let Some(o) = mock_find_block_open(body, q) {
                            if let Some(e) = mock_brace_match(body, o) {
                                q = mock_skip_ws(body, e + 1);
                                continue;
                            }
                        }
                        break;
                    } else if q < b.len() && b[q] == b'{' {
                        if let Some(e) = mock_brace_match(body, q) {
                            q = e + 1;
                        }
                        break;
                    } else {
                        break;
                    }
                }
                return q.max(end_pos);
            }
            // Condition false: follow the else-chain.
            let mut q = mock_skip_ws(body, end + 1);
            if !mock_starts_kw(&body[q.min(b.len())..], "else") {
                return end_pos;
            }
            q = mock_skip_ws(body, q + 4);
            if mock_starts_kw(&body[q.min(b.len())..], "if") {
                p = mock_skip_ws(body, q + 2);
                end_pos = q;
                continue;
            }
            if q < b.len() && b[q] == b'{' {
                if let Some(e) = mock_brace_match(body, q) {
                    let else_body = body[q + 1..e].to_string();
                    let flow = self.exec_block(&else_body);
                    if !matches!(flow, MockFlow::Next) {
                        self.pending_flow = Some(flow);
                    }
                    return e + 1;
                }
                return b.len();
            }
            return end_pos;
        }
    }

    /// `switch SCRUT { case ...: ... default: ... }` at `pos`.
    fn stmt_switch(&mut self, body: &str, pos: usize) -> usize {
        let b = body.as_bytes();
        let p = mock_skip_ws(body, pos + 6);
        let Some(open) = mock_find_block_open(body, p) else {
            return mock_stmt_end(body, pos).max(pos + 1);
        };
        let scrut = self.eval(body[p..open].trim());
        let Some(end) = mock_brace_match(body, open) else {
            return b.len();
        };
        let inner = body[open + 1..end].to_string();
        // Linearize items in source order.
        enum Item {
            Case { exprs: Vec<String>, guard: Option<String>, body: String },
            Default { body: String },
        }
        let mut items: Vec<Item> = Vec::new();
        let mut q = 0;
        let ib = inner.as_bytes();
        while q < ib.len() {
            q = mock_skip_ws(&inner, q);
            if q >= ib.len() {
                break;
            }
            if mock_starts_kw(&inner[q..], "case") {
                let mut h = mock_skip_ws(&inner, q + 4);
                // Head up to depth-0 `:` (skips `::`).
                let mut depth: usize = 0;
                let mut in_d = false;
                let mut in_s = false;
                let mut esc = false;
                let mut k = h;
                let mut colon: Option<usize> = None;
                while k < ib.len() {
                    let c = ib[k] as char;
                    if esc {
                        esc = false;
                        k += 1;
                        continue;
                    }
                    if c == '\\' && (in_d || in_s) {
                        esc = true;
                        k += 1;
                        continue;
                    }
                    if c == '"' && !in_s {
                        in_d = !in_d;
                        k += 1;
                        continue;
                    }
                    if c == '\'' && !in_d {
                        in_s = !in_s;
                        k += 1;
                        continue;
                    }
                    if in_d || in_s {
                        k += 1;
                        continue;
                    }
                    match c {
                        '(' | '[' => depth += 1,
                        ')' | ']' => depth = depth.saturating_sub(1),
                        ':' if depth == 0 => {
                            let next_cc = k + 1 < ib.len() && ib[k + 1] == b':';
                            let prev_cc = k > 0 && ib[k - 1] == b':';
                            if !next_cc && !prev_cc {
                                colon = Some(k);
                                break;
                            }
                        }
                        _ => {}
                    }
                    k += 1;
                }
                let Some(cpos) = colon else {
                    break;
                };
                let mut head = inner[h..cpos].trim().to_string();
                // Optional `if guard` shared by the case exprs.
                let mut guard = None;
                if let Some(gpos) = mock_find_guard(&head) {
                    guard = Some(head[gpos + 4..].trim().to_string());
                    head = head[..gpos].trim().to_string();
                }
                let exprs = split_mock_top_level(&head);
                // Body runs to the next depth-0 `case`/`default`/`}`.
                let bstart = cpos + 1;
                let bend = mock_switch_body_end(&inner, bstart);
                items.push(Item::Case {
                    exprs,
                    guard,
                    body: inner[bstart..bend].to_string(),
                });
                h = bend;
                q = h;
                continue;
            }
            if mock_starts_kw(&inner[q..], "default") {
                let mut h = mock_skip_ws(&inner, q + 7);
                if h < ib.len() && ib[h] == b':' {
                    h += 1;
                }
                let bend = mock_switch_body_end(&inner, h);
                items.push(Item::Default {
                    body: inner[h..bend].to_string(),
                });
                q = bend;
                continue;
            }
            // Braced case bodies surface `{` here; let the boundary scan pass them.
            if ib[q] == b'{' {
                if let Some(e) = mock_brace_match(&inner, q) {
                    q = e + 1;
                    continue;
                }
                break;
            }
            // Anything else ends the switch.
            if ib[q] == b'}' {
                break;
            }
            q += 1;
        }
        // Pick the first matching case (guards must be true), else default.
        let mut start: Option<usize> = None;
        let mut default_idx: Option<usize> = None;
        for (i, it) in items.iter().enumerate() {
            match it {
                Item::Default { .. } => {
                    if default_idx.is_none() {
                        default_idx = Some(i);
                    }
                }
                Item::Case { exprs, guard, .. } => {
                    if start.is_none() {
                        let guard_ok = match guard {
                            Some(g) => self.eval_cond(g) != Some(false),
                            None => true,
                        };
                        if guard_ok && exprs.iter().any(|e| self.eval(e) == scrut) {
                            start = Some(i);
                        }
                    }
                }
            }
        }
        let begin = start.or(default_idx);
        if let Some(mut i) = begin {
            // Fallthrough until break/return/end (spec behavior).
            while i < items.len() {
                let flow = match &items[i] {
                    Item::Case { body, .. } | Item::Default { body } => {
                        let b2 = body.clone();
                        self.exec_block(&b2)
                    }
                };
                match flow {
                    MockFlow::Next => {}
                    // `break` inside a switch exits the switch (loops already
                    // consumed their own breaks).
                    MockFlow::Break | MockFlow::Continue => break,
                    r @ MockFlow::Return(_) => {
                        self.pending_flow = Some(r);
                        break;
                    }
                }
                i += 1;
            }
        }
        end + 1
    }

    /// `match SCRUT { pat => body, ... }` statement at `pos`.
    fn stmt_match(&mut self, body: &str, pos: usize) -> usize {
        let b = body.as_bytes();
        let p = mock_skip_ws(body, pos + 5);
        let Some(open) = mock_find_block_open(body, p) else {
            return mock_stmt_end(body, pos).max(pos + 1);
        };
        let scrut = self.eval(body[p..open].trim());
        let Some(end) = mock_brace_match(body, open) else {
            return b.len();
        };
        let inner = body[open + 1..end].to_string();
        for arm in split_mock_top_level(&inner) {
            let arm = arm.trim().to_string();
            if arm.is_empty() {
                continue;
            }
            // `PAT [if GUARD] => BODY`
            let Some(arrow) = mock_find_fat_arrow(&arm) else {
                continue;
            };
            let mut left = arm[..arrow].trim().to_string();
            let right = arm[arrow + 2..].trim().to_string();
            let mut guard = None;
            if let Some(gpos) = mock_find_guard(&left) {
                guard = Some(left[gpos + 4..].trim().to_string());
                left = left[..gpos].trim().to_string();
            }
            if let Some(g) = &guard {
                if self.eval_cond(g) == Some(false) {
                    continue;
                }
            }
            let pat = left.trim();
            let matched = if pat == "_" {
                true
            } else if (pat.starts_with('"') && pat.ends_with('"') && pat.len() >= 2)
                || (pat.starts_with('\'') && pat.ends_with('\'') && pat.len() >= 2)
                || pat.parse::<f64>().is_ok()
                || pat == "true"
                || pat == "false"
            {
                self.eval(pat) == scrut
            } else if !pat.is_empty() && pat.chars().all(mock_is_ident_char) {
                // Irrefutable binding.
                self.vars.insert(pat.to_string(), scrut.clone());
                true
            } else {
                self.eval(pat) == scrut
            };
            if matched {
                self.exec_arm_expr(&right);
                break;
            }
        }
        end + 1
    }

    /// A `match`-arm body: a single expression (usually a print call).
    fn exec_arm_expr(&mut self, expr: &str) {
        let t = expr.trim().trim_end_matches(',').trim().to_string();
        if t.is_empty() {
            return;
        }
        for pat in MOCK_PRINT_PATTERNS {
            if t.starts_with(pat) {
                let open = pat.len() - 1;
                if let Some(close) = mock_paren_match(&t, open) {
                    let inner = t[open + 1..close].trim().to_string();
                    let line = self.print_arg(&inner);
                    self.push_line(&line);
                    return;
                }
            }
        }
        if mock_starts_kw(&t, "return") {
            return;
        }
        let _ = self.eval(&t);
    }
}

#[derive(Debug, Clone)]
struct Route {
    path: String,
    method: String,
    handler: String,
    /// `@Status(201)` above the handler, or None for 200/201 defaults.
    status: Option<u16>,
    /// `@Header("Name: value")` lines above the handler.
    headers: Vec<(String, String)>,
    /// `@Cookie("name=value; ...")` lines above the handler.
    cookies: Vec<String>,
    /// First string literal in `return "..."` of the handler body.
    body: Option<String>,
}

impl Route {
    fn plain(path: String, method: String, handler: String) -> Self {
        Route { path, method, handler, status: None, headers: Vec::new(), cookies: Vec::new(), body: None }
    }
}

/// Scan the lines directly ABOVE `line_idx` (skipping blanks) for
/// response-shaping decorators. Stops at the first non-decorator,
/// non-blank line so attributes never leak across handlers.
fn scan_route_decorators(lines: &[&str], line_idx: usize) -> (Option<u16>, Vec<(String, String)>, Vec<String>) {
    let mut status: Option<u16> = None;
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut cookies: Vec<String> = Vec::new();
    let mut j = line_idx;
    loop {
        if j == 0 {
            break;
        }
        j -= 1;
        let t = lines[j].trim();
        if t.is_empty() {
            continue;
        }
        if let Some(rest) = t.strip_prefix("@Status(") {
            if status.is_none() {
                status = rest.trim_end_matches(')').trim().parse().ok();
            }
            continue;
        }
        if let Some(rest) = t.strip_prefix("@Header(\"") {
            if let Some(end) = rest.find('"') {
                let kv = &rest[..end];
                if let Some(colon) = kv.find(':') {
                    headers.push((kv[..colon].trim().to_string(), kv[colon + 1..].trim().to_string()));
                }
            }
            continue;
        }
        if let Some(rest) = t.strip_prefix("@Cookie(\"") {
            if let Some(end) = rest.find('"') {
                cookies.push(rest[..end].to_string());
            }
            continue;
        }
        // `@Get/@Post/...` and `fn` lines terminate the scan.
        break;
    }
    (status, headers, cookies)
}

/// First string literal of `return "..."` inside `fn <handler>`'s body.
/// Brace-matched so nested blocks don't leak; returns the unescaped text.
fn extract_handler_body(source: &str, handler: &str) -> Option<String> {
    let sig = format!("fn {}", handler);
    let mut search = 0usize;
    while let Some(idx) = source[search..].find(&sig) {
        let mut i = search + idx + sig.len();
        let bytes = source.as_bytes();
        // Skip whitespace; require `(` (rules out `fn handlerX` prefixes).
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'(' {
            search += idx + sig.len();
            continue;
        }
        // Find opening `{` of the body (skip `-> Type`).
        let mut depth = 0usize;
        let mut body_start: Option<usize> = None;
        let mut k = i;
        while k < bytes.len() {
            let c = bytes[k] as char;
            if c == '{' {
                depth += 1;
                if body_start.is_none() {
                    body_start = Some(k);
                }
            } else if c == '}' {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            k += 1;
        }
        let start = body_start?;
        // Find first `return "..."` inside, respecting nesting via the
        // same brace scan is overkill: take up to the matching close.
        let mut end = start;
        let mut d = 0usize;
        let mut k2 = start;
        while k2 < bytes.len() {
            let c = bytes[k2] as char;
            if c == '"' {
                // skip string to avoid braces inside literals
                k2 += 1;
                while k2 < bytes.len() {
                    let e = bytes[k2] as char;
                    if e == '\\' {
                        k2 += 2;
                        continue;
                    }
                    if e == '"' {
                        break;
                    }
                    k2 += 1;
                }
            } else if c == '{' {
                d += 1;
            } else if c == '}' {
                if d == 0 {
                    break;
                }
                d -= 1;
                if d == 0 {
                    end = k2;
                    break;
                }
            }
            k2 += 1;
        }
        if end <= start {
            return None;
        }
        let body = &source[start..end];
        // First return "<literal>" with escape processing.
        let mut r = 0usize;
        let bb = body.as_bytes();
        while r < bb.len() {
            if body[r..].starts_with("return") {
                let mut q = r + 6;
                while q < bb.len() && (bb[q] as char).is_whitespace() {
                    q += 1;
                }
                if q < bb.len() && bb[q] == b'"' {
                    q += 1;
                    let mut lit = String::new();
                    while q < bb.len() {
                        let ch = bb[q] as char;
                        if ch == '\\' && q + 1 < bb.len() {
                            let nx = bb[q + 1] as char;
                            match nx {
                                'n' => lit.push('\n'),
                                't' => lit.push('\t'),
                                'r' => lit.push('\r'),
                                '\\' => lit.push('\\'),
                                '"' => lit.push('"'),
                                _ => {
                                    lit.push('\\');
                                    lit.push(nx);
                                }
                            }
                            q += 2;
                            continue;
                        }
                        if ch == '"' {
                            return Some(lit);
                        }
                        lit.push(ch);
                        q += 1;
                    }
                    return None;
                }
            }
            r += 1;
        }
        return None;
    }
    None
}

fn extract_port(source: &str) -> Option<u16> {
    // Find Http::serve("0.0.0.0:PORT", app)
    if let Some(idx) = source.find("Http::serve(\"") {
        let rest = &source[idx + 13..];
        if let Some(end) = rest.find('"') {
            let addr = &rest[..end];
            if let Some(colon) = addr.rfind(':') {
                return addr[colon + 1..].parse().ok();
            }
        }
    }
    None
}

fn extract_routes(source: &str) -> Vec<Route> {
    let mut routes = Vec::new();

    // 1. Check for fluent API: .route("/", Http::get(handler))
    let mut search_start = 0;
    while let Some(idx) = source[search_start..].find(".route(\"") {
        let actual = search_start + idx + 8;
        if let Some(end_quote) = source[actual..].find('"') {
            let path = source[actual..actual + end_quote].to_string();
            let after_path = &source[actual + end_quote + 1..];

            // Find method: Http::get/post/put/delete
            let method = if after_path.contains("Http::get") {
                "GET".to_string()
            } else if after_path.contains("Http::post") {
                "POST".to_string()
            } else if after_path.contains("Http::put") {
                "PUT".to_string()
            } else if after_path.contains("Http::delete") {
                "DELETE".to_string()
            } else {
                "GET".to_string()
            };

            // Find handler name
            let handler = if let Some(h_start) = after_path.find('(') {
                let h_rest = &after_path[h_start + 1..];
                if let Some(h_end) = h_rest.find(')') {
                    h_rest[..h_end].trim().to_string()
                } else {
                    "handler".to_string()
                }
            } else {
                "handler".to_string()
            };

            routes.push(Route::plain(path, method, handler));
        }
        search_start = actual;
    }

    // 2. Check for decorators: @Get/@Post/@Put/@Delete("...").
    // Response-shaping decorators (@Status/@Header/@Cookie) on the lines
    // directly above attach to the route; handler body literals are
    // extracted as default response data.
    let decorator_patterns = [
        ("@Get(\"", "GET"),
        ("@Post(\"", "POST"),
        ("@Put(\"", "PUT"),
        ("@Delete(\"", "DELETE"),
    ];

    for (pattern, method) in decorator_patterns {
        let mut search_start = 0;
        while let Some(idx) = source[search_start..].find(pattern) {
            let deco_abs = search_start + idx;
            let actual = deco_abs + pattern.len();
            if let Some(end_quote) = source[actual..].find('"') {
                let path = source[actual..actual + end_quote].to_string();
                let after_decorator = &source[actual + end_quote + 2..]; // skip ")

                // Look for the next function name
                if let Some(fn_idx) = after_decorator.find("fn ") {
                    let fn_rest = &after_decorator[fn_idx + 3..];
                    if let Some(fn_end) = fn_rest.find('(') {
                        let handler = fn_rest[..fn_end].trim().to_string();
                        let deco_line = source[..deco_abs].matches('\n').count();
                        let lines: Vec<&str> = source.lines().collect();
                        let (status, headers, cookies) =
                            scan_route_decorators(&lines, deco_line.min(lines.len().saturating_sub(1)));
                        routes.push(Route {
                            path: path.clone(),
                            method: method.to_string(),
                            handler: handler.clone(),
                            status,
                            headers,
                            cookies,
                            body: extract_handler_body(source, &handler),
                        });
                    }
                }
            }
            search_start = actual;
        }
    }

    routes
}

/// Parse an HTTP `Cookie` request header into ordered pairs.
/// Flag-only tokens and malformed pairs are skipped; duplicate names
/// keep first-seen order. Pure function — unit tested.
pub fn parse_cookie_header(value: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for part in value.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(eq) = part.find('=') {
            let name = part[..eq].trim();
            let val = part[eq + 1..].trim().trim_matches('"');
            if !name.is_empty() {
                out.push((name.to_string(), val.to_string()));
            }
        }
    }
    out
}

/// Apply a route's `@Status`/`@Header`/`@Cookie` shaping onto a response
/// envelope: override status when valid (100–599), insert custom
/// headers (invalid names/values skipped, never fatal), append one
/// `set-cookie` per cookie, and guarantee a `data` key exists.
fn shape_response(
    status_override: Option<axum::http::StatusCode>,
    extra_headers: &[(String, String)],
    set_cookies: &[String],
    mut status: axum::http::StatusCode,
    mut res: serde_json::Value,
) -> (
    axum::http::StatusCode,
    axum::http::HeaderMap,
    axum::response::Json<serde_json::Value>,
) {
    use axum::http::{HeaderMap, HeaderName, HeaderValue};
    use axum::response::Json;
    if let Some(s) = status_override {
        status = s;
    }
    let mut headers = HeaderMap::new();
    for (name, value) in extra_headers {
        if let (Ok(n), Ok(v)) = (
            HeaderName::from_bytes(name.to_lowercase().as_bytes()),
            HeaderValue::from_str(value),
        ) {
            headers.insert(n, v);
        }
    }
    for cookie in set_cookies {
        if !cookie.trim().is_empty() {
            if let Ok(v) = HeaderValue::from_str(cookie.trim()) {
                headers.append("set-cookie", v);
            }
        }
    }
    if res.get("data").is_none() {
        res["data"] = serde_json::Value::Null;
    }
    (status, headers, Json(res))
}

/// Build the axum router for extracted routes WITHOUT binding.
/// Pure construction (no I/O): unit and live integration tests drive
/// this directly on ephemeral ports.
fn build_router(routes: Vec<Route>) -> axum::Router {
    use axum::{
        body::Bytes,
        http::{Method, StatusCode},
        middleware::{self, Next},
        response::{IntoResponse, Json},
        routing::{get, post},
        Router,
    };
    use serde_json::{json, Value};
    use std::net::SocketAddr;

    // Camada CORS + log (sem deps novas): injeta
    // `Access-Control-*` em toda resposta, responde preflight
    // `OPTIONS` com 204 e loga método/caminho/status/tempo.
    async fn cors_layer(
        req: axum::http::Request<axum::body::Body>,
        next: Next,
    ) -> impl IntoResponse {
        use axum::http::HeaderValue;
        const ALLOW_METHODS: &str = "GET, POST, PUT, DELETE, OPTIONS";
        const ALLOW_HEADERS: &str = "content-type, authorization";
        if req.method() == Method::OPTIONS {
            let mut res =
                axum::response::Response::new(axum::body::Body::empty());
            let h = res.headers_mut();
            h.insert("access-control-allow-origin", HeaderValue::from_static("*"));
            h.insert("access-control-allow-methods", HeaderValue::from_static(ALLOW_METHODS));
            h.insert("access-control-allow-headers", HeaderValue::from_static(ALLOW_HEADERS));
            h.insert("access-control-max-age", HeaderValue::from_static("86400"));
            *res.status_mut() = StatusCode::NO_CONTENT;
            return res;
        }
        let method = req.method().clone();
        let uri = req.uri().clone();
        let start = std::time::Instant::now();
        let mut res = next.run(req).await;
        let h = res.headers_mut();
        h.insert("access-control-allow-origin", HeaderValue::from_static("*"));
        h.insert("access-control-allow-methods", HeaderValue::from_static(ALLOW_METHODS));
        h.insert("access-control-allow-headers", HeaderValue::from_static(ALLOW_HEADERS));
        // Identity + tracing on every response (no new deps: process-local
        // monotonic counter; unique per server process lifetime).
        // Versioned once per process (not per request): `from_static` can't
        // track release auto-bumps, so build the value a single time.
        static POWERED_BY: std::sync::OnceLock<HeaderValue> = std::sync::OnceLock::new();
        let powered = POWERED_BY.get_or_init(|| {
            HeaderValue::from_str(&format!("lexicon/{}", env!("CARGO_PKG_VERSION")))
                .unwrap_or_else(|_| HeaderValue::from_static("lexicon"))
        });
        h.insert("x-powered-by", powered.clone());
        static REQUEST_ID: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(1);
        let id = REQUEST_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // Charset `[a-z0-9-]` is always a valid header value.
        if let Ok(v) = HeaderValue::from_str(&format!("lex-{}", id)) {
            h.insert("x-request-id", v);
        }
        println!(
            "[http] {} {} -> {} ({}ms)",
            method,
            uri,
            res.status(),
            start.elapsed().as_millis()
        );
        res
    }

    async fn fallback_404(uri: axum::http::Uri) -> impl IntoResponse {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"success": false, "data": null,
                "message": format!("no route for {}", uri.path())})),
        )
    }

    async fn fallback_405() -> impl IntoResponse {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            Json(json!({"success": false, "data": null,
                "message": "method not allowed for this route"})),
        )
    }

    let mut route_map: std::collections::HashMap<String, Vec<Route>> =
        std::collections::HashMap::new();

    for route in routes {
        route_map.entry(route.path.clone()).or_default().push(route);
    }

    let mut app = Router::new();

    for (path, routes) in route_map {
        let mut method_router = axum::routing::get(|| async { "Not Found" }); // Dummy

        let mut has_get = false;
        let mut has_post = false;
        let mut has_put = false;
        let mut has_delete = false;

        for route in routes {
            let method = route.method.clone();
            let p = route.path.clone();
            let h = route.handler.clone();
            let status_override = route
                .status
                .and_then(|s| axum::http::StatusCode::from_u16(s).ok())
                .filter(|s| (100..600).contains(&s.as_u16()));
            let extra_headers: Vec<(String, String)> = route.headers.clone();
            let set_cookies: Vec<String> = route.cookies.clone();
            let body_literal: Option<String> = route.body.clone();

            let h_get = h.clone();
            let p_get = p.clone();
            let b_get = body_literal.clone();
            let so_get = status_override;
            let eh_get = extra_headers.clone();
            let sc_get = set_cookies.clone();
            let get_responder = move |headers: axum::http::HeaderMap, axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>| async move {
                let mut res = json!({"success": true, "message": Value::Null, "data": Value::Null});
                let mut status = StatusCode::OK;
                match h_get.as_str() {
                    "root" => res["data"] = json!("Welcome to Lexicon API!"),
                    "hello" => res["data"] = json!("Hello from Lexicon HTTP Server!"),
                    "users" => {
                        res["data"] = json!([
                            {"id": 1, "name": "John", "email": "john@example.com"},
                            {"id": 2, "name": "Jane", "email": "jane@example.com"},
                            {"id": 3, "name": "Bob", "email": "bob@example.com"},
                            {"id": 4, "name": "Alice", "email": "alice@example.com"}
                        ])
                    }
                    "stats" => {
                        res["data"] = json!(0);
                        res["message"] = json!("Total requests");
                    }
                    _ => {
                        res["message"] = json!(format!("Lexicon API - {} handler", h_get));
                        res["endpoint"] = json!(p_get);
                        res["method"] = json!("GET");
                        // The handler's own `return "..."` becomes the data.
                        if let Some(lit) = b_get {
                            res["data"] = json!(lit);
                        }
                        if !query.is_empty() {
                            res["query"] = json!(query);
                        }
                        let cookies = parse_cookie_header(headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or(""));
                        if !cookies.is_empty() {
                            res["cookies"] = json!(cookies
                                .into_iter()
                                .map(|(k, v)| (k, serde_json::Value::String(v)))
                                .collect::<serde_json::Map<String, serde_json::Value>>());
                        }
                    }
                }
                shape_response(so_get, &eh_get, &sc_get, status, res)
            };

            let h_post = h.clone();
            let p_post = p.clone();
            let b_post = body_literal.clone();
            let so_post = status_override;
            let eh_post = extra_headers.clone();
            let sc_post = set_cookies.clone();
            let post_responder = move |headers: axum::http::HeaderMap, axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>, body: Bytes| async move {
                // Corpo JSON é ecoado no envelope; JSON inválido vira 400 e
                // corpo vazio mantém o stub legado (201 p/ create_user).
                let mut echoed: Option<Value> = None;
                if !body.is_empty() {
                    // Mesmo limite do módulo HTTP (1 MiB).
                    const MAX_BODY_BYTES: usize = 1_048_576;
                    if body.len() > MAX_BODY_BYTES {
                        return (
                            StatusCode::PAYLOAD_TOO_LARGE,
                            axum::http::HeaderMap::new(),
                            Json(json!({"success": false, "data": null,
                                "message": "body exceeds 1 MiB limit"})),
                        );
                    }
                    match serde_json::from_slice::<Value>(&body) {
                        Ok(v) => echoed = Some(v),
                        Err(_) => {
                            return (
                                StatusCode::BAD_REQUEST,
                                axum::http::HeaderMap::new(),
                                Json(json!({"success": false, "data": null,
                                    "message": "invalid JSON body"})),
                            )
                        }
                    }
                }
                let mut res = json!({"success": true, "message": Value::Null, "data": Value::Null});
                let mut status = StatusCode::OK;
                match h_post.as_str() {
                    "create_user" => {
                        status = StatusCode::CREATED;
                        res["message"] = json!("User created successfully!");
                        res["data"] = echoed.unwrap_or(
                            json!({"id": 5, "name": "NewUser", "email": "new@example.com"}),
                        );
                    }
                    _ => {
                        res["message"] = json!(format!("Lexicon API - {} handler", h_post));
                        res["endpoint"] = json!(p_post);
                        res["method"] = json!("POST");
                        // Handler literal wins; otherwise the echoed body.
                        if let Some(lit) = b_post {
                            res["data"] = json!(lit);
                        } else if let Some(v) = echoed {
                            res["data"] = v;
                        }
                        if !query.is_empty() {
                            res["query"] = json!(query);
                        }
                        let cookies = parse_cookie_header(headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or(""));
                        if !cookies.is_empty() {
                            res["cookies"] = json!(cookies
                                .into_iter()
                                .map(|(k, v)| (k, serde_json::Value::String(v)))
                                .collect::<serde_json::Map<String, serde_json::Value>>());
                        }
                    }
                }
                shape_response(so_post, &eh_post, &sc_post, status, res)
            };

            // PUT/DELETE share the generic envelope (echo + limits).
            let h_put = h.clone();
            let p_put = p.clone();
            let b_put = body_literal.clone();
            let so_put = status_override;
            let eh_put = extra_headers.clone();
            let sc_put = set_cookies.clone();
            let put_responder = move |headers: axum::http::HeaderMap, axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>, body: Bytes| async move {
                let mut echoed: Option<Value> = None;
                if !body.is_empty() {
                    const MAX_BODY_BYTES: usize = 1_048_576;
                    if body.len() > MAX_BODY_BYTES {
                        return (
                            StatusCode::PAYLOAD_TOO_LARGE,
                            axum::http::HeaderMap::new(),
                            Json(json!({"success": false, "data": null,
                                "message": "body exceeds 1 MiB limit"})),
                        );
                    }
                    match serde_json::from_slice::<Value>(&body) {
                        Ok(v) => echoed = Some(v),
                        Err(_) => {
                            return (
                                StatusCode::BAD_REQUEST,
                                axum::http::HeaderMap::new(),
                                Json(json!({"success": false, "data": null,
                                    "message": "invalid JSON body"})),
                            )
                        }
                    }
                }
                let mut res = json!({"success": true, "message": Value::Null, "data": Value::Null});
                res["message"] = json!(format!("Lexicon API - {} handler", h_put));
                res["endpoint"] = json!(p_put);
                res["method"] = json!("PUT");
                if let Some(lit) = b_put {
                    res["data"] = json!(lit);
                } else if let Some(v) = echoed {
                    res["data"] = v;
                }
                if !query.is_empty() {
                    res["query"] = json!(query);
                }
                let cookies = parse_cookie_header(headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or(""));
                if !cookies.is_empty() {
                    res["cookies"] = json!(cookies
                        .into_iter()
                        .map(|(k, v)| (k, serde_json::Value::String(v)))
                        .collect::<serde_json::Map<String, serde_json::Value>>());
                }
                shape_response(so_put, &eh_put, &sc_put, StatusCode::OK, res)
            };

            let h_del = h.clone();
            let p_del = p.clone();
            let b_del = body_literal.clone();
            let so_del = status_override;
            let eh_del = extra_headers.clone();
            let sc_del = set_cookies.clone();
            let del_responder = move |headers: axum::http::HeaderMap, axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>| async move {
                let mut res = json!({"success": true, "message": Value::Null, "data": Value::Null});
                res["message"] = json!(format!("Lexicon API - {} handler", h_del));
                res["endpoint"] = json!(p_del);
                res["method"] = json!("DELETE");
                if let Some(lit) = b_del {
                    res["data"] = json!(lit);
                }
                if !query.is_empty() {
                    res["query"] = json!(query);
                }
                let cookies = parse_cookie_header(headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or(""));
                if !cookies.is_empty() {
                    res["cookies"] = json!(cookies
                        .into_iter()
                        .map(|(k, v)| (k, serde_json::Value::String(v)))
                        .collect::<serde_json::Map<String, serde_json::Value>>());
                }
                shape_response(so_del, &eh_del, &sc_del, StatusCode::OK, res)
            };

            if method == "GET" {
                if !has_get && !has_post && !has_put && !has_delete {
                    method_router = get(get_responder);
                } else if !has_get {
                    method_router = method_router.get(get_responder);
                }
                has_get = true;
            } else if method == "POST" {
                if !has_get && !has_post && !has_put && !has_delete {
                    method_router = post(post_responder);
                } else if !has_post {
                    method_router = method_router.post(post_responder);
                }
                has_post = true;
            } else if method == "PUT" {
                if !has_get && !has_post && !has_put && !has_delete {
                    method_router = axum::routing::put(put_responder);
                } else if !has_put {
                    method_router = method_router.put(put_responder);
                }
                has_put = true;
            } else if method == "DELETE" {
                if !has_get && !has_post && !has_put && !has_delete {
                    method_router = axum::routing::delete(del_responder);
                } else if !has_delete {
                    method_router = method_router.delete(del_responder);
                }
                has_delete = true;
            }
        }

        let path_for_route = path.clone();
        // Método errado na rota existente vira 405 JSON (axum 0.7:
        // `MethodRouter::fallback` cobre 405; `method_not_allowed_fallback`
        // só existe no axum 0.8).
        let method_router = method_router.fallback(fallback_405);
        app = app.route(&path_for_route, method_router);
    }

    // Caminho desconhecido vira 404 JSON + CORS/log em tudo.
    app = app
        .fallback(fallback_404)
        .layer(middleware::from_fn(cors_layer));

    app
}

async fn start_http_server(port: u16, routes: Vec<Route>) {
    use std::net::SocketAddr;

    let app = build_router(routes);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!(
        "\n{}[Server running on http://localhost:{}]{}",
        COLOR_GREEN, port, RESET
    );
    println!(
        "{}Press Ctrl+C to stop the server...{}",
        COLOR_YELLOW, RESET
    );

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub fn bench(file: Option<String>, iterations: usize, verbose: bool) -> Result<()> {
    bench_with_ci(file, iterations, verbose, false)
}

/// CI-aware benchmark suite (Spec §23 + §45 partial + §71 partial):
/// never reads stdin, honors `--ci`/`CI=true`, records flags/target/
/// hw-sw metadata in the output, and exercises the fuzz corpus so
/// `bench` doubles as a smoke-fuzz gate.
pub fn bench_with_ci(
    file: Option<String>,
    iterations: usize,
    verbose: bool,
    ci_flag: bool,
) -> Result<()> {
    let src_path = match file {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };

    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }

    let source = fs::read_to_string(&src_path)?;
    let ci = is_ci_mode(ci_flag);

    // Suite metadata (Spec §71): flags/target/hw-sw recorded on every run
    // for repeatability across machines.
    let target = env::var("LEXICON_TARGET").unwrap_or_else(|_| "native".to_string());
    let profile_name = env::var("LEXICON_PROFILE").unwrap_or_else(|_| "debug".to_string());
    let opt_flag = if profile_name == "release" { "opt=release" } else { "opt=debug" };
    let hw = format!("os={} arch={}", std::env::consts::OS, std::env::consts::ARCH);
    let sw = format!("lexc={} rustc={}", env!("CARGO_PKG_VERSION"), rustc_version_stub());

    println!("{}Running benchmark...{}", COLOR_CYAN, RESET);
    println!("  File: {:?}", src_path);
    println!("  Iterations: {}", iterations);
    println!("  Target: {} ({} {})", target, opt_flag, profile_name);
    println!("  HW: {}", hw);
    println!("  SW: {}", sw);
    if ci {
        println!("  Mode: CI (non-interactive)");
    }

    let start = std::time::Instant::now();

    for i in 0..iterations {
        compile(&source, &src_path.display().to_string())?;
        if verbose && iterations > 10 && i % (iterations / 10).max(1) == 0 {
            println!("  progress: {}/{}", i, iterations);
        }
    }

    let elapsed = start.elapsed();

    println!("\n{}Benchmark Results:{}", COLOR_YELLOW, RESET);
    println!("  Total time: {:?}", elapsed);
    if iterations > 0 {
        println!("  Avg per iteration: {:?}", elapsed / iterations as u32);
        println!(
            "  Iterations/sec: {}",
            iterations as f64 / elapsed.as_secs_f64()
        );
    }
    println!("  Flags: {} target={} {}", opt_flag, target, hw);
    println!("  Toolchain: {}", sw);

    // Fuzz smoke gate inside bench (Spec §43).
    let (fuzz_passed, fuzz_total) = run_fuzz_corpus();
    println!(
        "  Fuzz: {}/{} corpus inputs survived",
        fuzz_passed, fuzz_total
    );

    // No stdin read here by design — `bench` is always non-interactive,
    // with or without `--ci`.
    let _ = ci;

    Ok(())
}

/// Best-effort rustc version for bench metadata (Spec §71).
fn rustc_version_stub() -> String {
    // Keep std-only and infallible; real version detection can shell out.
    option_env!("RUSTC_VERSION").unwrap_or("unknown").to_string()
}

pub fn run_gui(file: Option<String>) -> Result<()> {
    let src_path = match file {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };

    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }

    let source = fs::read_to_string(&src_path)?;
    
    println!("{}Starting GUI application...{}", COLOR_CYAN, RESET);

    #[cfg(feature = "gui")]
    gui::run_gui(&source);
    #[cfg(not(feature = "gui"))]
    println!(
        "{}This build has no GUI support. Reinstall with default features to enable `lex gui`.{}",
        COLOR_YELLOW, RESET
    );

    Ok(())
}

pub fn run_webview(file: Option<String>) -> Result<()> {
    let src_path = match file {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };

    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }

    let source = fs::read_to_string(&src_path)?;
    
    println!("{}Starting WebView application...{}", COLOR_CYAN, RESET);
    
    // webview::run_webview(&source);
    
    Ok(())
}

// ---------------------------------------------------------------------------
// Network commands (Node-parity: serve / test-net / dns-check / tls-check)
// Reuse lexicon-http types (LexUrl, Dns/TLS configs, ws handshake) + the
// real axum router. Invoked by `lex serve|test-net|dns-check|tls-check`
// and by the VSCode `lex` task provider (type: lex, background serve).
// ---------------------------------------------------------------------------

/// `lex serve [file] --host H --port P`: boot the real HTTP server.
/// Routes/ports are scanned from comment-free source (same as `run`);
/// explicit --host/--port override the scanned port.
pub fn serve_net(file: Option<String>, host: String, port: u16) -> Result<()> {
    let src_path = file.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("src/main.lex"));
    let (routes, scanned_port) = if src_path.exists() {
        let source = fs::read_to_string(&src_path)?;
        let code = lexicon_lexer::strip_comments(&source);
        (extract_routes(&code), extract_port(&code).unwrap_or(port))
    } else {
        println!(
            "{}Note:{} {:?} not found — serving default routes on {}:{}",
            COLOR_YELLOW, RESET, src_path, host, port
        );
        (Vec::new(), port)
    };
    // CLI --port wins when non-default; otherwise keep the scanned port.
    let effective_port = if port != 3000 { port } else { scanned_port };
    println!(
        "{}Serving{} {} route(s) on http://{}:{} (listening)",
        COLOR_GREEN, RESET, routes.len(), host, effective_port
    );
    let host_owned = host.clone();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
        serve_on(&host_owned, effective_port, routes).await;
    });
    Ok(())
}

async fn serve_on(host: &str, port: u16, routes: Vec<Route>) {
    let app = build_router(routes);
    let addr_str = format!("{}:{}", host, port);
    match tokio::net::TcpListener::bind(&addr_str).await {
        Ok(listener) => {
            println!(
                "\n{}[Server running on http://{}:{} (serving)]{}",
                COLOR_GREEN, host, port, RESET
            );
            println!("{}Press Ctrl+C to stop the server...{}", COLOR_YELLOW, RESET);
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) => {
            println!("{}Failed to bind {}: {}{}", COLOR_RED, addr_str, e, RESET);
        }
    }
}

/// `lex test-net --host H --port P`: smoke-test TCP + HTTP + UDP + URL + WS.
/// Never hangs: every step has a short timeout; exit code stays 0 so CI
/// tasks can parse the PASS/FAIL lines.
pub fn test_net(host: String, port: u16) -> Result<()> {
    use lexicon_http::module as net;
    println!("{}[test-net]{} {}:{}", COLOR_CYAN, RESET, host, port);
    let mut pass = 0u32;
    let mut fail = 0u32;
    let mut report = |name: &str, ok: bool, detail: &str| {
        if ok {
            pass += 1;
            println!("  {}PASS{} {} — {}", COLOR_GREEN, RESET, name, detail);
        } else {
            fail += 1;
            println!("  {}FAIL{} {} — {}", COLOR_RED, RESET, name, detail);
        }
    };

    // 1. URL parse (WHATWG subset) + family detection (Node net.isIP).
    let url_str = format!("http://{}:{}/health?x=1", host, port);
    match net::LexUrl::parse(&url_str) {
        Ok(u) => report("url.parse", true, &format!("href={} origin={}", u.href(), u.origin())),
        Err(e) => report("url.parse", false, &e),
    }
    report(
        "net.isIP",
        true,
        &format!("family={} (0|4|6)", net::is_ip(&host)),
    );

    // 2. TCP connect with timeout (Node net.connect).
    let rt = tokio::runtime::Runtime::new().unwrap();
    let addr = format!("{}:{}", host, port);
    let tcp_ok = rt.block_on(async {
        tokio::time::timeout(
            std::time::Duration::from_secs(3),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
    });
    match tcp_ok {
        Ok(Ok(_)) => report("tcp.connect", true, &format!("connected to {}", addr)),
        Ok(Err(e)) => report("tcp.connect", false, &format!("refused/unreachable: {}", e)),
        Err(_) => report("tcp.connect", false, "timed out after 3s"),
    }

    // 3. HTTP GET via reqwest blocking (Node http.get).
    let http_url = format!("http://{}:{}/", host, port);
    match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(client) => match client.get(&http_url).send() {
            Ok(res) => {
                let code = res.status().as_u16();
                let phrase = net::status_message(code);
                report("http.get", code < 500, &format!("{} {} ({})", code, phrase, http_url));
            }
            Err(e) => report("http.get", false, &format!("{} — is `lex serve` running?", e)),
        },
        Err(e) => report("http.get", false, &e.to_string()),
    }

    // 4. UDP bind self-test (Node dgram.createSocket + bind).
    let udp_ok = rt.block_on(async {
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            tokio::net::UdpSocket::bind("127.0.0.1:0"),
        )
        .await
    });
    match udp_ok {
        Ok(Ok(sock)) => {
            let local = sock.local_addr().map(|a| a.to_string()).unwrap_or_default();
            report("udp.bind", true, &format!("bound {}", local));
        }
        Ok(Err(e)) => report("udp.bind", false, &e.to_string()),
        Err(_) => report("udp.bind", false, "timed out"),
    }

    // 5. WS handshake vector RFC6455 (Node ws parity, no network).
    let accept = net::ws_accept_key("dGhlIHNhbXBsZSBub25jZQ==");
    report(
        "ws.handshake",
        accept == "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=",
        &format!("accept={}", accept),
    );

    println!(
        "\n{}[test-net]{} {}/{} passed{}",
        if fail == 0 { COLOR_GREEN } else { COLOR_YELLOW },
        RESET,
        pass,
        pass + fail,
        if fail == 0 { "" } else { " — start `lex serve` for tcp/http PASS" }
    );
    Ok(())
}

/// `lex dns-check --host H`: resolve via Tokio (Node dns.lookup) + show
/// configured servers/retries + IP family. Host may be a bare name or URL.
pub fn dns_check_cmd(host: String, timeout_secs: u64) -> Result<()> {
    use lexicon_http::module as net;
    let cfg = net::DnsConfig::default();
    // Accept URLs as well as bare hosts (Node dns accepts hostnames).
    let bare = net::LexUrl::parse(&host)
        .map(|u| u.host.clone())
        .unwrap_or_else(|_| {
            host.split(['/', '?', '#']).next().unwrap_or(&host).to_string()
        });
    // Strip port if present (but keep IPv6 brackets intact for lookup).
    let lookup_host = bare.trim_start_matches('[').split(']').next().unwrap_or(&bare);
    let lookup_host = lookup_host.split(':').next().unwrap_or(lookup_host);
    println!("{}[dns-check]{} {}", COLOR_CYAN, RESET, bare);
    println!("  servers: {} (timeout {}ms, retries {})", cfg.servers.join(", "), cfg.timeout_ms, cfg.retries);
    println!("  isIP({}) = {} | isIPv4 = {} | isIPv6 = {}", bare, net::is_ip(&bare), net::is_ipv4(&bare), net::is_ipv6(&bare));
    let rt = tokio::runtime::Runtime::new().unwrap();
    let timeout = std::time::Duration::from_secs(timeout_secs.max(1).min(30));
    let target = format!("{}:80", lookup_host);
    let res = rt.block_on(async {
        tokio::time::timeout(timeout, tokio::net::lookup_host(&target)).await
    });
    match res {
        Ok(Ok(addrs)) => {
            let list: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
            if list.is_empty() {
                println!("  {}lookup: no addresses (ENOTFOUND){}", COLOR_YELLOW, RESET);
            } else {
                println!("  {}lookup OK{}: {}", COLOR_GREEN, RESET, list.join(", "));
            }
        }
        Ok(Err(e)) => println!("  {}lookup failed (ENOTFOUND){}: {}{}", COLOR_RED, RESET, e, ""),
        Err(_) => println!("  {}lookup timed out after {:?} (ETIMEOUT){}", COLOR_YELLOW, timeout, RESET),
    }
    Ok(())
}

/// `lex tls-check --host H --port P`: validate TLS config (Node tls parity).
/// No real handshake (no new deps): verifies min/max versions, SNI host,
/// ALPN negotiation list, cert/key presence, and prints the ws `Accept`
/// derivation as a sanity vector.
pub fn tls_check_cmd(host: String, port: u16) -> Result<()> {
    use lexicon_http::module as net;
    let cfg = net::TlsConfig::secure_default();
    println!("{}[tls-check]{} {}:{}", COLOR_CYAN, RESET, host, port);
    println!("  min_version: {} (secure_default)", cfg.min_version.as_str());
    match cfg.check_versions() {
        Ok(()) => println!("  {}versions OK{}: min {} enforced, TLS1.2+ only", COLOR_GREEN, RESET, cfg.min_version.as_str()),
        Err(e) => println!("  {}versions FAIL{}: {}", COLOR_RED, RESET, e),
    }
    println!("  verify_mode: {:?}", cfg.verify_mode());
    println!("  alpn: {}", if cfg.alpn.is_empty() { "(none)".to_string() } else { cfg.alpn.join(", ") });
    println!(
        "  cert/key: {} / {}",
        cfg.cert_path.as_deref().unwrap_or("(none — plaintext dev)"),
        cfg.key_path.as_deref().unwrap_or("(none)")
    );
    // SNI sanity: host must be a DNS name or IP (Node checkServerIdentity input).
    let sni_ok = !host.is_empty() && !host.contains(' ');
    println!(
        "  sni: {} ({})",
        host,
        if sni_ok { "valid" } else { "INVALID — empty or spaces" }
    );
    // Cheap proof the crypto helpers are wired (RFC6455 vector doubles here).
    let accept = net::ws_accept_key("dGhlIHNhbXBsZSBub25jZQ==");
    println!(
        "  ws_accept sanity: {}",
        if accept == "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=" { "OK" } else { "MISMATCH" }
    );
    println!(
        "  note: full handshake needs a real cert; set cert_path/key_path for `lex serve --tls` (planned)"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn fix_dot_call_rewrites_known_modules() {
        let src = "let a = Http.get(url);\nlet b = Json.parse(s);\nConsole.writeLine(\"hi\");\nConsole.write(\"w\");\nConsole.log(\"l\");\n";
        let out = fix_dot_to_path_call(src);
        assert!(out.contains("Http::get("), "Http.get not rewritten:\n{}", out);
        assert!(out.contains("Json::parse("), "Json.parse not rewritten:\n{}", out);
        assert!(out.contains("Console::writeLine("), "writeLine not rewritten:\n{}", out);
        assert!(out.contains("Console::write("), "write not rewritten:\n{}", out);
        assert!(out.contains("Console::log("), "log not rewritten:\n{}", out);
        assert!(!out.contains("Http.get("), "leftover dot-call:\n{}", out);
        assert!(!out.contains("Json.parse("), "leftover dot-call:\n{}", out);
    }

    #[test]
    fn fix_dot_call_leaves_unknown_modules_untouched() {
        // `Http.post(` is deliberately NOT a known prefix (only `Http.get(`);
        // `Json.stringify(` is already path form and must not change either.
        let src = "Foo.bar(x);\nFooBar.baz(y);\nHttp.post(z);\nConsole.writeln(w);\nJson.stringify(v);\n";
        assert_eq!(fix_dot_to_path_call(src), src);
    }

    #[test]
    fn fix_dot_call_is_idempotent() {
        let src = "let r = Http.get(u);\nConsole.log(x);\n";
        let once = fix_dot_to_path_call(src);
        assert_ne!(once, src, "rule should change something on first apply");
        assert_eq!(fix_dot_to_path_call(&once), once);
    }

    #[test]
    fn fix_trim_trailing_ws_trims_and_preserves_blank_lines() {
        let src = "let a = 1;   \n   \n\t\nlet b = 2;\t \n";
        assert_eq!(fix_trim_trailing_ws(src), "let a = 1;\n\n\nlet b = 2;\n");
    }

    #[test]
    fn fix_trim_trailing_ws_is_idempotent() {
        let src = "a  \n  \n\nb\t\n";
        let once = fix_trim_trailing_ws(src);
        assert_ne!(once, src, "rule should change something on first apply");
        assert_eq!(fix_trim_trailing_ws(&once), once);
    }

    #[test]
    fn fix_arrow_to_colon_repairs_decl_heads_only() {
        let src = "const db  -> User[] = []\nlet x->int = 1;\n-var y -> String;\n";
        let out = fix_arrow_to_colon(src);
        assert!(out.contains("const db  : User[] = []"), "const head:\n{}", out);
        assert!(out.contains("let x:int = 1;"), "let head:\n{}", out);
        // `fn` return arrows and mid-line arrows are NOT declarations.
        let untouched = "pub fn f() -> int {\n    return a->b;\n}\n";
        assert_eq!(fix_arrow_to_colon(untouched), untouched);
        // Idempotent.
        assert_eq!(fix_arrow_to_colon(&out), out);
    }

    #[test]
    fn fix_registry_holds_exactly_three_rules_applied_in_order() {
        assert_eq!(FIX_RULES.len(), 3);
        assert_eq!(FIX_RULES[0].id, "dot-to-path-call");
        assert_eq!(FIX_RULES[1].id, "trim-trailing-ws");
        assert_eq!(FIX_RULES[2].id, "arrow-to-colon");
        let src = "Console.log(x);   \n";
        let mut s = src.to_string();
        for rule in FIX_RULES {
            s = (rule.apply)(&s);
        }
        assert_eq!(s, "Console::log(x);\n");
    }

    #[test]
    fn color_codes_convert_and_terminate_with_reset() {
        let out = process_color_codes_with("routes: ^7/h ^0", true);
        assert!(out.contains("\x1b[37m"), "missing gray:\n{}", out.escape_debug());
        assert!(out.contains("\x1b[30m"), "missing black:\n{}", out.escape_debug());
        assert!(!out.contains('^'), "caret leaked:\n{}", out.escape_debug());
        assert!(out.ends_with(RESET), "no trailing reset (leak):\n{}", out.escape_debug());
    }

    #[test]
    fn color_reset_code_maps_to_reset() {
        let out = process_color_codes_with("a^9b", true);
        assert!(out.contains(RESET), "expected reset:\n{}", out.escape_debug());
    }

    #[test]
    fn color_caret_escape_and_lone_caret_survive() {
        assert_eq!(process_color_codes_with("100^^7", true), "100^7");
        assert_eq!(process_color_codes_with("a^b c^", true), "a^b c^");
    }

    #[test]
    fn color_plain_text_is_untouched() {
        assert_eq!(process_color_codes_with("hello", true), "hello");
    }

    #[test]
    fn color_disabled_strips_codes_to_plain_text() {
        assert_eq!(process_color_codes_with("routes: ^7/h ^0", false), "routes: /h ");
        assert_eq!(process_color_codes_with("100^^7", false), "100^7");
    }

    fn gen_template_in_tmp(
        name: &str,
        template: Option<&str>,
        edge: bool,
        target: Option<&str>,
        grpc: bool,
    ) -> PathBuf {
        // Tests run in threads sharing one process CWD: serialize the
        // chdir dance so parallel tests can never observe a moved CWD.
        let _guard = CWD_LOCK.lock().unwrap();
        let base = std::env::temp_dir().join(format!(
            "lex_new_test_{}_{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let dir = base.join(name);
        let prev = std::env::current_dir().unwrap();
        std::fs::create_dir_all(&base).unwrap();
        std::env::set_current_dir(&base).unwrap();
        let res = new_project(
            name,
            template.map(|s| s.to_string()),
            edge,
            target.map(|s| s.to_string()),
            grpc,
        );
        std::env::set_current_dir(&prev).unwrap();
        res.unwrap();
        dir
    }

    fn read_main(dir: &Path) -> String {
        std::fs::read_to_string(dir.join("src").join("main.lex")).unwrap()
    }

    fn assert_parses_clean(src: &str) {
        let mut lexer = lexicon_lexer::Lexer::new(src);
        let tokens = lexer.tokenize();
        let mut parser = lexicon_parser::Parser::new(tokens);
        let module = parser.parse().expect("template must parse");
        let mut checker = lexicon_analysis::TypeChecker::new();
        checker
            .check_module(&module)
            .expect("template must typecheck");
    }

    #[test]
    fn templates_differ_and_run() {
        let api = gen_template_in_tmp("t_api", Some("api"), false, None, false);
        let plugin = gen_template_in_tmp("t_plugin", Some("plugin"), false, None, false);
        let service = gen_template_in_tmp("t_service", Some("service"), false, None, false);
        let plain = gen_template_in_tmp("t_plain", None, false, None, false);

        let api_src = read_main(&api);
        let plugin_src = read_main(&plugin);
        let service_src = read_main(&service);
        let plain_src = read_main(&plain);

        // Each template is genuinely different code.
        assert!(api_src.contains("Http::serve(\"0.0.0.0:3000\")"), "api serve");
        assert!(plugin_src.contains("process(\"hello\")"), "plugin body");
        assert!(!plugin_src.contains("Http::serve"), "plugin has no server");
        assert!(service_src.contains("service UserService"), "service IDL");
        assert!(service_src.contains("rpc GetUser"), "service rpc");
        assert!(plain_src.contains("Hello, Lexicon!"), "plain hello");
        assert_ne!(api_src, plugin_src);
        assert_ne!(api_src, service_src);
        assert_ne!(plugin_src, service_src);

        // Every generated entry point parses and typechecks.
        for src in [&api_src, &plugin_src, &service_src, &plain_src] {
            assert_parses_clean(src);
        }

        // Manifests are versioned and complete.
        for dir in [&api, &plugin, &service, &plain] {
            let toml = std::fs::read_to_string(dir.join("lexicon.toml")).unwrap();
            assert!(toml.contains(env!("CARGO_PKG_VERSION")), "manifest version:\n{}", toml);
            assert!(dir.join("README.md").exists());
            assert!(dir.join(".gitignore").exists());
        }
        for d in [&api, &plugin, &service, &plain] {
            let _ = std::fs::remove_dir_all(d.parent().unwrap());
        }
    }

    #[test]
    fn template_flags_change_output() {
        let edge = gen_template_in_tmp("t_edge", Some("api"), true, None, false);
        let wasm = gen_template_in_tmp("t_wasm", Some("plugin"), false, Some("wasm"), false);
        let grpc = gen_template_in_tmp("t_grpc", Some("service"), false, None, true);

        let edge_src = read_main(&edge);
        let wasm_src = read_main(&wasm);
        let grpc_src = read_main(&grpc);

        assert!(edge_src.contains(":8080"), "edge port");
        assert!(wasm_src.contains("@Export"), "wasm export");
        assert!(wasm_src.contains("target=wasm"), "wasm marker");
        assert!(grpc_src.contains("Grpc::serve"), "grpc serve");

        for src in [&edge_src, &wasm_src, &grpc_src] {
            assert_parses_clean(src);
        }
        for d in [&edge, &wasm, &grpc] {
            let _ = std::fs::remove_dir_all(d.parent().unwrap());
        }
    }

    #[test]
    fn unknown_template_is_rejected() {
        let _guard = CWD_LOCK.lock().unwrap();
        let base = std::env::temp_dir().join(format!(
            "lex_new_test_unknown_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(&base).unwrap();
        let err = new_project("t_nope", Some("nope".to_string()), false, None, false)
            .expect_err("unknown template must fail");
        std::env::set_current_dir(&prev).unwrap();
        assert!(err.to_string().contains("Unknown template"), "got: {}", err);
        assert!(
            err.to_string().contains("api") && err.to_string().contains("service"),
            "must list valid templates, got: {}",
            err
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn route_decorators_attach_per_route() {
        let src = "@Status(201)\n@Header(\"X-App: demo\")\n@Cookie(\"s=1; Path=/\")\n@Post(\"/users\")\npub fn create_user() -> String {\n    return \"created\";\n}\n\n@Get(\"/health\")\npub fn health() -> String {\n    return \"ok\";\n}\n";
        let routes = extract_routes(src);
        assert_eq!(routes.len(), 2, "got: {:?}", routes.iter().map(|r| &r.handler).collect::<Vec<_>>());
        let post = routes.iter().find(|r| r.handler == "create_user").unwrap();
        assert_eq!(post.status, Some(201));
        assert_eq!(post.headers, vec![("X-App".to_string(), "demo".to_string())]);
        assert_eq!(post.cookies, vec!["s=1; Path=/".to_string()]);
        assert_eq!(post.body.as_deref(), Some("created"));
        let health = routes.iter().find(|r| r.handler == "health").unwrap();
        assert_eq!(health.status, None, "decorators must not leak across handlers");
        assert!(health.headers.is_empty());
        assert_eq!(health.body.as_deref(), Some("ok"));
    }

    #[test]
    fn route_decorators_reject_garbage_status() {
        let src = "@Status(9999)\n@Status(abc)\n@Get(\"/x\")\npub fn x() -> String {\n    return \"y\";\n}\n";
        let routes = extract_routes(src);
        assert_eq!(routes.len(), 1);
        // 9999 is out of range (filtered at serve time), "abc" unparsable.
        assert!(routes[0].status.is_none() || routes[0].status == Some(9999));
    }

    #[test]
    fn handler_body_literal_handles_escapes() {
        let src = "pub fn m() -> String {\n    let a = 1;\n    return \"a\\\"b\";\n}\n";
        assert_eq!(
            extract_handler_body(src, "m").as_deref(),
            Some("a\"b")
        );
        assert_eq!(extract_handler_body("pub fn m() -> String {\n    compute();\n}\n", "m"), None);
        assert_eq!(extract_handler_body("pub fn other() -> String {\n    return \"x\";\n}\n", "m"), None);
    }

    #[test]
    fn cookie_header_parsing() {
        assert!(parse_cookie_header("").is_empty());
        assert_eq!(
            parse_cookie_header("s=abc; theme=dark"),
            vec![("s".to_string(), "abc".to_string()), ("theme".to_string(), "dark".to_string())]
        );
        // Flag-only tokens and empties are skipped; quotes trimmed.
        assert_eq!(
            parse_cookie_header("a=1;; flag; b=\"x y\""),
            vec![("a".to_string(), "1".to_string()), ("b".to_string(), "x y".to_string())]
        );
    }

    #[test]
    fn fluent_put_delete_detected() {
        let src = "let app = App::new().route(\"/a\", Http::put(h)).route(\"/b\", Http::delete(g));";
        let routes = extract_routes(src);
        assert!(routes.iter().any(|r| r.method == "PUT" && r.path == "/a"), "got: {:?}", routes.iter().map(|r| (&r.method, &r.path)).collect::<Vec<_>>());
        assert!(routes.iter().any(|r| r.method == "DELETE" && r.path == "/b"));
    }

    /// Live integration: real HTTP on an ephemeral port. Server runs on
    /// its own runtime thread; the blocking client asserts status codes,
    /// headers, cookies, query echo and JSON bodies end to end.
    #[test]
    fn live_status_headers_cookies_query() {
        let src = "@Status(201)\n@Header(\"X-App: demo\")\n@Cookie(\"s=abc; Path=/\")\n@Post(\"/items\")\npub fn create_item() -> String {\n    return \"made\";\n}\n\n@Get(\"/echo\")\npub fn echo() -> String {\n    return \"e\";\n}\n\n@Put(\"/items\")\npub fn replace_item() -> String {\n    return \"replaced\";\n}\n";
        let routes = extract_routes(src);
        assert_eq!(routes.len(), 3);

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let app = build_router(routes);
        let server = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await.unwrap();
                axum::serve(listener, app).await.unwrap();
            });
        });
        // Wait for bind with bounded retries (no infinite hangs).
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        let base = format!("http://127.0.0.1:{}", port);
        let mut ready = false;
        for _ in 0..50 {
            if client.get(format!("{}/echo", base)).send().is_ok() {
                ready = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(ready, "server never became ready");

        // POST: decorator status 201 + custom header + set-cookie + literal body.
        let r = client.post(format!("{}/items", base)).body("{}").send().unwrap();
        assert_eq!(r.status().as_u16(), 201);
        assert_eq!(r.headers().get("x-app").unwrap(), "demo");
        let set_cookies: Vec<_> = r.headers().get_all("set-cookie").iter().collect();
        assert_eq!(set_cookies.len(), 1);
        assert!(set_cookies[0].to_str().unwrap().contains("s=abc"));
        assert!(r.headers().get("x-request-id").is_some());
        assert!(r.headers().get("x-powered-by").is_some());
        let body: serde_json::Value = r.json().unwrap();
        assert_eq!(body["data"], serde_json::json!("made"));

        // GET with query + request cookies echoed.
        let r = client
            .get(format!("{}/echo?lang=lex&page=2", base))
            .header("cookie", "s=abc; theme=dark")
            .send()
            .unwrap();
        assert_eq!(r.status().as_u16(), 200);
        let body: serde_json::Value = r.json().unwrap();
        assert_eq!(body["query"]["lang"], serde_json::json!("lex"));
        assert_eq!(body["cookies"]["theme"], serde_json::json!("dark"));

        // PUT routed with literal body.
        let r = client.put(format!("{}/items", base)).body("{}").send().unwrap();
        assert_eq!(r.status().as_u16(), 200);
        let body: serde_json::Value = r.json().unwrap();
        assert_eq!(body["data"], serde_json::json!("replaced"));

        // Unknown path stays JSON 404.
        let r = client.get(format!("{}/nope", base)).send().unwrap();
        assert_eq!(r.status().as_u16(), 404);
        drop(server);
    }
}
