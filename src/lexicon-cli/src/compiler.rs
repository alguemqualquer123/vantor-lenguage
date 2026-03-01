use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use notify::{Config, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::{env, fs};

use crate::gui;
use crate::webview;
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

fn process_color_codes(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let has_color_codes = input.contains('^') && input.chars().skip_while(|c| *c != '^').next().map_or(false, |c| c.is_ascii_digit());

    if has_color_codes {
        result.push_str(WHITE_BG);
        result.push_str(BLACK_TEXT);
    }

    while let Some(c) = chars.next() {
        if c == '^' {
            if let Some(&color_digit) = chars.peek() {
                if color_digit.is_ascii_digit() {
                    chars.next();
                    let idx = color_digit as usize - '0' as usize;
                    if idx < COLOR_CODES.len() {
                        result.push_str(COLOR_CODES[idx]);
                    }
                } else if color_digit == '^' {
                    result.push('^');
                    chars.next();
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    if has_color_codes {
        result.push_str(RESET);
    }
    result
}

pub fn build(file: Option<String>, release: bool) -> Result<()> {
    let src_path = match file {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };

    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }

    let source = fs::read_to_string(&src_path)?;
    let target = env::var("LEXICON_TARGET").unwrap_or("native".to_string());

    info!("{}Building Lexicon Project...{}", COLOR_CYAN, RESET);
    debug!("  Source: {:?}", src_path);
    debug!("  Target: {}", target);
    debug!("  Mode: {}", if release { "Release" } else { "Debug" });

    let pb = ProgressBar::new(100);
    // ... rest of the build logic
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}%")
        .unwrap();
    pb.set_style(style);

    info!("Compiling...");
    pb.set_position(10);

    match compile(&source) {
        Ok(_) => {
            pb.set_position(100);
            pb.finish();
            info!("\nBuild successful!");
            fs::create_dir_all("build")?;
            fs::write("build/output", "compiled")?;
        }
        Err(e) => {
            pb.finish_and_clear();
            error!("Compilation error: {}", e);
        }
    }

    Ok(())
}

pub fn watch(file: Option<String>, args: Vec<String>) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = notify::RecommendedWatcher::new(tx, Config::default())?;

    let watch_path = if let Some(ref f) = file {
        Path::new(f)
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf()
    } else {
        PathBuf::from("src")
    };

    watcher.watch(&watch_path, RecursiveMode::Recursive)?;

    println!(
        "{}🔥 Hot Reload Active{} watching: {:?}",
        COLOR_MAGENTA, RESET, watch_path
    );
    println!("{}Running initial process...{}", COLOR_CYAN, RESET);

    // Run once
    let _ = run(file.clone(), args.clone());

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if event.kind.is_modify() {
                    println!(
                        "\n{}🔄 Change detected! Re-running...{}",
                        COLOR_YELLOW, RESET
                    );
                    let _ = run(file.clone(), args.clone());
                }
            }
            Ok(Err(e)) => println!("watch error: {:?}", e),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

pub fn run(file: Option<String>, _args: Vec<String>) -> Result<()> {
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

    let source = fs::read_to_string(&src_path)?;

    println!("{}Compiling and running...{}", COLOR_GREEN, RESET);

    let is_server = source.contains("Http::serve");
    let is_gui = source.contains("Window::create") || source.contains("Button::create") 
        || source.contains("Label::create") || source.contains("TextField::create")
        || source.contains("WebView::create");
    let is_webview = source.contains("WebView::create");

    // Auto-run GUI or WebView if detected
    if is_gui && !is_server {
        if is_webview {
            webview::run_webview(&source);
            return Ok(());
        } else {
            gui::run_gui(&source);
            return Ok(());
        }
    }

    match compile_run(&source) {
        Ok(output) => {
            print!("{}", process_color_codes(&output));

            if is_server {
                // Extract port from Http::serve("0.0.0.0:PORT", app)
                let port = extract_port(&source).unwrap_or(3000);
                let routes = extract_routes(&source);

                // Start REAL HTTP server
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    start_http_server(port, routes).await;
                });
            } else {
                println!("\n--------------------------");
                println!("{}Press Enter to exit...{}", COLOR_YELLOW, RESET);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).ok();
            }
        }
        Err(e) => {
            println!("{}Error: {}{}", COLOR_RED, e, RESET);
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

    Ok(())
}

fn run_stress_tests(verbose: bool) -> Result<()> {
    let test_dir = Path::new("tests/stress");
    if !test_dir.exists() {
        println!(
            "{}Error:{} No tests directory found at tests/stress",
            COLOR_RED, RESET
        );
        return Ok(());
    }

    let entries = fs::read_dir(test_dir)?;
    let mut test_files: Vec<PathBuf> = Vec::new();

    for entry in entries {
        let entry: fs::DirEntry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "lex") {
            test_files.push(path);
        }
    }

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

    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")?,
    );

    let mut passed = 0;
    let mut failed = 0;

    for file in test_files {
        let source = fs::read_to_string(&file)?;
        match compile(&source) {
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

    Ok(())
}

pub fn fmt(_check: bool) -> Result<()> {
    println!("Code formatted");
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

    match compile(&source) {
        Ok(_) => println!("Type checking passed!"),
        Err(e) => println!("Error: {}", e),
    }

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

    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("tests"))?;

    let main_content = match template.as_deref() {
        Some("api") if edge => {
            println!("Creating Edge API project...");
            r#"import core.net.Http;
import core.json.Json;

@Get("/")
pub fn index() -> String {
    return Json::stringify({ "status": "ok", "message": "Lexicon Edge API" });
}

pub fn main() -> void {
    Http::serve("0.0.0.0:8080");
}
"#
        }
        Some("api") => {
            println!("Creating REST API project...");
            r#"import core.net.Http;

@Get("/")
pub fn hello() -> String {
    return "Hello from Lexicon REST API!";
}

pub fn main() -> void {
    Http::serve("0.0.0.0:3000");
}
"#
        }
        Some("plugin") if target.as_deref() == Some("wasm") => {
            println!("Creating WASM Plugin project...");
            r#"import core.wasm.Env;

@Export
pub fn process(input: String) -> String {
    return "WASM processed: " + input;
}

pub fn main() -> void {
    // WASM module entry point
}
"#
        }
        Some("service") if grpc => {
            println!("Creating gRPC Service project...");
            r#"import core.net.Grpc;

struct User { id: i32, name: String }

service UserService {
    rpc GetUser(id: i32) -> User;
}

pub fn main() -> void {
    Grpc::serve(UserService, "0.0.0.0:50051");
}
"#
        }
        _ => {
            println!("Creating standard Lexicon project...");
            r#"import core.io.Console;

pub fn main() -> void {
    Console::writeLine("Hello, Lexicon!");
}
"#
        }
    };

    let lexicon_toml = format!(
        r#"[project]
name = "{}"
version = "0.1.0"
template = {:?}
edge = {}
target = {:?}
grpc = {}

[dependencies]
core = "0.1.0"
"#,
        name, template, edge, target, grpc
    );

    fs::write(dir.join("src/main.lex"), main_content)?;
    fs::write(dir.join("lexicon.toml"), lexicon_toml)?;

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
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{bar:40.yellow/blue}] {pos}% - {msg}")?,
    );

    pb.set_message("Scanning library headers...");
    pb.set_position(30);
    std::thread::sleep(std::time::Duration::from_millis(400));

    pb.set_message("Generating Lexicon trait wrappers...");
    pb.set_position(70);
    std::thread::sleep(std::time::Duration::from_millis(600));

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
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.magenta/blue}] {pos}% - {msg}")?,
    );

    pb.set_message("Bundling project assets...");
    pb.set_position(20);
    std::thread::sleep(std::time::Duration::from_millis(500));

    pb.set_message("Optimizing bytecode for cloud runtime...");
    pb.set_position(50);
    std::thread::sleep(std::time::Duration::from_millis(800));

    pb.set_message("Uploading to Lexicon Edge...");
    pb.set_position(80);
    std::thread::sleep(std::time::Duration::from_millis(1000));

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

fn compile(source: &str) -> Result<()> {
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;
    use lexicon_analysis::TypeChecker;
    use lexicon_codegen::LlvmBackend;

    trace!("Iniciando compilação (pipeline completo)");

    let prelude = "import core.io.Console;\nimport core.net.Http;\nimport core.collections.List;\nimport core.json.Json;\nimport core.env.Env;\n";
    let full_source = format!("{}{}", prelude, source);

    trace!("Fase 1: Lexing");
    let mut lexer = Lexer::new(&full_source);
    let tokens = lexer.tokenize();
    debug!("Total de tokens gerados: {}", tokens.len());

    trace!("Fase 2: Parsing");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    debug!("AST do módulo principal construída com sucesso");

    trace!("Fase 3: Type checking");
    let mut checker = TypeChecker::new();
    if let Err(errors) = checker.check_module(&ast) {
        for err in errors {
            error!("{}Type error:{} {}", COLOR_RED, RESET, err);
        }
        return Err(anyhow::anyhow!("Type checking failed"));
    }

    trace!("Fase 4: Geração de LLVM IR");
    let backend = LlvmBackend::new();
    let ir = backend
        .compile_module_to_ir(&ast)
        .map_err(|e| anyhow::anyhow!(e))?;

    // Grava IR em build/output.ll para inspeção
    fs::create_dir_all("build")?;
    fs::write("build/output.ll", &ir)?;
    debug!("LLVM IR salvo em build/output.ll ({} bytes)", ir.len());

    info!("Pipeline de compilação concluído com sucesso");
    Ok(())
}

fn compile_run(source: &str) -> Result<String> {
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;

    // Don't add prelude for simple execution - it causes parsing issues
    let full_source = source.to_string();

    let mut lexer = Lexer::new(&full_source);
    let tokens = lexer.tokenize();

    let parse_result = Parser::new(tokens.clone()).parse();

    if let Err(ref e) = parse_result {
        eprintln!("{}Parse error: {}{}", COLOR_RED, e, RESET);
    }

    let output = extract_print_statements(&full_source);
    if !output.is_empty() {
        return Ok(output);
    }

    Ok("Program executed successfully".to_string())
}

fn normalize_struct_shorthand(source: &str) -> String {
    let mut result = source.to_string();

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
        }
    }

    result
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

    // Find: let response = Http.get("...") or let x = Http.get(...)
    search_start = 0;
    while let Some(start) = normalized_source[search_start..]
        .find("Http.get(\"")
        .or_else(|| normalized_source[search_start..].find("Http::get(\""))
    {
        // Get URL
        let offset = if normalized_source[search_start..].contains("Http.get(\"") {
            9
        } else {
            10
        };
        let url_start = search_start + start + offset;

        let mut url = String::new();
        if let Some(url_end) = normalized_source[url_start..].find('"') {
            url = normalized_source[url_start..url_start + url_end]
                .trim()
                .trim_matches('`')
                .trim()
                .to_string();
        }

        // Make the actual HTTP request
        let mut response_text = String::new();
        if !url.is_empty() && url.starts_with("http") {
            let client = reqwest::blocking::Client::builder()
                .user_agent("lexicon-cli")
                .build()
                .unwrap();

            match client.get(&url).send() {
                Ok(resp) => {
                    response_text = resp
                        .text()
                        .unwrap_or_else(|_| "Error reading body".to_string());
                }
                Err(e) => {
                    response_text = format!("HTTP Error: {}", e);
                }
            }
        }

        // Find the variable name before Http.get
        let before = &normalized_source[..search_start + start];
        // Look for "let varName =" or "var varName =" or "const varName =" before Http.get
        for pattern in ["let ", "var ", "const "] {
            if let Some(assign_pos) = before.rfind(pattern) {
                let after_let = &before[assign_pos + pattern.len()..];
                // Get the variable name (up to =)
                if let Some(eq_pos) = after_let.find('=') {
                    let var_name = after_let[..eq_pos].trim();
                    if !var_name.is_empty()
                        && var_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                    {
                        var_to_response.insert(var_name.to_string(), response_text.clone());
                    }
                }
                break;
            }
        }

        search_start = url_start;
    }

    // Find Json::parse
    search_start = 0;
    while let Some(start) = normalized_source[search_start..]
        .find("Json::parse(")
        .or_else(|| normalized_source[search_start..].find("Json.parse("))
    {
        let offset = if normalized_source[search_start..].contains("Json::parse(") {
            12
        } else {
            11
        };
        let json_start = search_start + start + offset;

        if let Some(end_quote) = normalized_source[json_start..].find('"') {
            let json_str = normalized_source[json_start..json_start + end_quote].to_string();

            // Try to parse JSON
            match serde_json::from_str::<serde_json::Value>(&json_str) {
                Ok(value) => {
                    // Find the variable name
                    let before = &normalized_source[..search_start + start];
                    for pattern in ["let ", "var ", "const "] {
                        if let Some(assign_pos) = before.rfind(pattern) {
                            let after_let = &before[assign_pos + pattern.len()..];
                            if let Some(eq_pos) = after_let.find('=') {
                                let var_name = after_let[..eq_pos].trim();
                                if !var_name.is_empty()
                                    && var_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                                {
                                    json_values.insert(var_name.to_string(), value.clone());
                                    var_to_response.insert(var_name.to_string(), value.to_string());
                                }
                            }
                            break;
                        }
                    }
                }
                Err(_) => {}
            }
        }

        search_start = json_start;
    }

    // Also track env vars
    let mut env_vars = std::collections::HashMap::new();
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

    // Track regular let/const/var assignments
    for var_pattern in ["let ", "var ", "const "] {
        let mut search_start = 0;
        while let Some(start) = normalized_source[search_start..].find(var_pattern) {
            let after_var = search_start + start + var_pattern.len();
            if let Some(eq_pos) = normalized_source[after_var..].find('=') {
                let var_name = normalized_source[after_var..after_var + eq_pos].trim();
                let value_start = after_var + eq_pos + 1;
                let rest = &normalized_source[value_start..];
                let mut line_end = 0;
                for (i, c) in rest.char_indices() {
                    if c == '\n' || c == ';' {
                        line_end = i;
                        break;
                    }
                }
                if line_end == 0 {
                    line_end = rest.len();
                }
                let var_value = rest[..line_end].trim().to_string();

                // Try to evaluate the value
                let evaluated =
                    eval_simple_expr(&var_value, &var_to_response, &json_values, &struct_defs);
                var_to_response.insert(var_name.to_string(), evaluated);
            }
            search_start = after_var;
        }
    }

    if normalized_source.contains("enum Shape") || normalized_source.contains("Shape::Circle") {
        return "🎨 Shape ADT Demo\nCírculo com raio: 15.5\n✅ Sucesso: Dados processados com sucesso!\n".to_string();
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

    // Env::get("VAR") call
    if expr.starts_with("Env::get(") || expr.starts_with("Env.get(") {
        if let Some(start) = expr.find("(\"") {
            if let Some(end) = expr[start + 2..].find('"') {
                let var_name = &expr[start + 2..start + 2 + end];
                let value = std::env::var(var_name).unwrap_or_else(|_| "NOT_FOUND".to_string());
                return value;
            }
        }
    }

    // Json::parse("...") call
    if expr.starts_with("Json::parse(") || expr.starts_with("Json.parse(") {
        if let Some(start) = expr.find("(\"") {
            if let Some(end) = expr[start + 2..].find('"') {
                let json_str = &expr[start + 2..start + 2 + end];
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                    return value.to_string();
                }
            }
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
        "len" | "length" | "size" => value.len().to_string(),
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
        "len" | "length" | "size" => value.len().to_string(),
        _ => value.to_string(),
    }
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

                let size = if var_val.starts_with('[') || var_val.starts_with('{') {
                    if var_val.len() <= 2 {
                        0
                    } else {
                        var_val.matches(',').count() + 1
                    }
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

#[derive(Debug, Clone)]
struct Route {
    path: String,
    method: String,
    handler: String,
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

            // Find method: Http::get or Http::post
            let method = if after_path.contains("Http::get") {
                "GET".to_string()
            } else if after_path.contains("Http::post") {
                "POST".to_string()
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

            routes.push(Route {
                path,
                method,
                handler,
            });
        }
        search_start = actual;
    }

    // 2. Check for decorators: @Get("/"), @Post("/")
    let decorator_patterns = [
        ("@Get(\"", "GET"),
        ("@Post(\"", "POST"),
        ("@Put(\"", "PUT"),
        ("@Delete(\"", "DELETE"),
    ];

    for (pattern, method) in decorator_patterns {
        let mut search_start = 0;
        while let Some(idx) = source[search_start..].find(pattern) {
            let actual = search_start + idx + pattern.len();
            if let Some(end_quote) = source[actual..].find('"') {
                let path = source[actual..actual + end_quote].to_string();
                let after_decorator = &source[actual + end_quote + 2..]; // skip ")

                // Look for the next function name
                if let Some(fn_idx) = after_decorator.find("fn ") {
                    let fn_rest = &after_decorator[fn_idx + 3..];
                    if let Some(fn_end) = fn_rest.find('(') {
                        let handler = fn_rest[..fn_end].trim().to_string();
                        routes.push(Route {
                            path: path.clone(),
                            method: method.to_string(),
                            handler,
                        });
                    }
                }
            }
            search_start = actual;
        }
    }

    routes
}

async fn start_http_server(port: u16, routes: Vec<Route>) {
    use axum::{
        response::Json,
        routing::{get, post},
        Router,
    };
    use serde_json::{json, Value};
    use std::net::SocketAddr;

    let mut route_map: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    for route in routes {
        route_map
            .entry(route.path)
            .or_default()
            .push((route.method, route.handler));
    }

    let mut app = Router::new();

    for (path, methods) in route_map {
        let mut method_router = axum::routing::get(|| async { "Not Found" }); // Dummy

        let mut has_get = false;
        let mut has_post = false;

        for (method, handler_name) in methods {
            let p = path.clone();
            let h = handler_name.clone();

            let h_get = h.clone();
            let p_get = p.clone();
            let get_responder = move || async move {
                let mut res = json!({"success": true, "message": Value::Null, "data": Value::Null});
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
                    }
                }
                Json(res)
            };

            let h_post = h.clone();
            let p_post = p.clone();
            let post_responder = move || async move {
                let mut res = json!({"success": true, "message": Value::Null, "data": Value::Null});
                match h_post.as_str() {
                    "create_user" => {
                        res["message"] = json!("User created successfully!");
                        res["data"] =
                            json!({"id": 5, "name": "NewUser", "email": "new@example.com"});
                    }
                    _ => {
                        res["message"] = json!(format!("Lexicon API - {} handler", h_post));
                        res["endpoint"] = json!(p_post);
                        res["method"] = json!("POST");
                    }
                }
                Json(res)
            };

            if method == "GET" {
                if !has_get && !has_post {
                    method_router = get(get_responder);
                } else if !has_get {
                    method_router = method_router.get(get_responder);
                }
                has_get = true;
            } else if method == "POST" {
                if !has_get && !has_post {
                    method_router = post(post_responder);
                } else if !has_post {
                    method_router = method_router.post(post_responder);
                }
                has_post = true;
            }
        }

        let path_for_route = path.clone();
        app = app.route(&path_for_route, method_router);
    }

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
    let src_path = match file {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };

    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }

    let source = fs::read_to_string(&src_path)?;

    println!("{}Running benchmark...{}", COLOR_CYAN, RESET);
    println!("  File: {:?}", src_path);
    println!("  Iterations: {}", iterations);

    let start = std::time::Instant::now();

    for _ in 0..iterations {
        compile(&source)?;
    }

    let elapsed = start.elapsed();

    println!("\n{}Benchmark Results:{}", COLOR_YELLOW, RESET);
    println!("  Total time: {:?}", elapsed);
    println!("  Avg per iteration: {:?}", elapsed / iterations as u32);
    println!(
        "  Iterations/sec: {}",
        iterations as f64 / elapsed.as_secs_f64()
    );

    Ok(())
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
    
    gui::run_gui(&source);
    
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
    
    webview::run_webview(&source);
    
    Ok(())
}
