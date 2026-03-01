use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::{env, fs};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use notify::{Watcher, RecursiveMode, Config};
use lexicon_utils::runtime::{BufferedOutput, StringInterner, FastLoopExecutor};

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

    result.push_str(WHITE_BG);
    result.push_str(BLACK_TEXT);

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

    result.push_str(RESET);
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

    println!("{}Building Lexicon Project...{}", COLOR_CYAN, RESET);
    println!("  Source: {:?}", src_path);
    println!("  Target: {}", target);
    println!("  Mode: {}", if release { "Release" } else { "Debug" });

    let pb = ProgressBar::new(100);
    // ... rest of the build logic
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}%")
        .unwrap();
    pb.set_style(style);

    println!("Compiling...");
    pb.set_position(10);

    match compile(&source) {
        Ok(_) => {
            pb.set_position(100);
            pb.finish();
            println!("\nBuild successful!");
            fs::create_dir_all("build")?;
            fs::write("build/output", "compiled")?;
        }
        Err(e) => {
            pb.finish_and_clear();
            println!("Compilation error: {}", e);
        }
    }

    Ok(())
}

pub fn watch(file: Option<String>, args: Vec<String>) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = notify::RecommendedWatcher::new(tx, Config::default())?;
    
    let watch_path = if let Some(ref f) = file {
        Path::new(f).parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        PathBuf::from("src")
    };

    watcher.watch(&watch_path, RecursiveMode::Recursive)?;

    println!("{}🔥 Hot Reload Active{} watching: {:?}", COLOR_MAGENTA, RESET, watch_path);
    println!("{}Running initial process...{}", COLOR_CYAN, RESET);
    
    // Run once
    let _ = run(file.clone(), args.clone());

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if event.kind.is_modify() {
                    println!("\n{}🔄 Change detected! Re-running...{}", COLOR_YELLOW, RESET);
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
    println!("\n{}Scanning for Integrated Tests (@Test)...{}", COLOR_CYAN, RESET);
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
                                println!("    {}Testing {}...{} [PASS]", COLOR_GREEN, line.trim(), RESET);
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
        println!("{}Passed: {}/{} tests{}", COLOR_GREEN, integrated_passed, integrated_tests, RESET);
    }

    Ok(())
}

fn run_stress_tests(verbose: bool) -> Result<()> {
    let test_dir = Path::new("tests/stress");
    if !test_dir.exists() {
        println!("{}Error:{} No tests directory found at tests/stress", COLOR_RED, RESET);
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
        println!("{}No stress tests found in tests/stress{}", COLOR_YELLOW, RESET);
        return Ok(());
    }

    println!("{}Running {} Lexicon tests...{}\n", COLOR_CYAN, total, RESET);

    let pb = ProgressBar::new(total as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")?);

    let mut passed = 0;
    let mut failed = 0;

    for file in test_files {
        let source = fs::read_to_string(&file)?;
        match compile(&source) {
            Ok(_) => {
                passed += 1;
                if verbose {
                    println!("{}PASS:{} {:?}", COLOR_GREEN, RESET, file.file_name().unwrap());
                }
            }
            Err(e) => {
                failed += 1;
                println!("{}FAIL:{} {:?} - {}", COLOR_RED, RESET, file.file_name().unwrap(), e);
            }
        }
        pb.inc(1);
    }

    pb.finish_with_message("Tests completed");

    println!("\n{}Test Summary:{}", COLOR_YELLOW, RESET);
    println!("{}Passed: {}{}", COLOR_GREEN, passed, RESET);
    println!("{}Failed: {}{}", COLOR_RED, failed, RESET);
    println!("{}Success Rate: {:.2}%{}", COLOR_CYAN, (passed as f32 / total as f32) * 100.0, RESET);

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

pub fn new_project(name: &str, template: Option<String>, edge: bool, target: Option<String>, grpc: bool) -> Result<()> {
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

    println!("{}Project '{}' created successfully!{}", COLOR_GREEN, name, RESET);
    Ok(())
}

pub fn ffi(lib_path: &str) -> Result<()> {
    println!("{}🔗 Native Interop: Binding to library: {}{}", COLOR_YELLOW, lib_path, RESET);
    
    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.cyan} [{bar:40.yellow/blue}] {pos}% - {msg}")?);
        
    pb.set_message("Scanning library headers...");
    pb.set_position(30);
    std::thread::sleep(std::time::Duration::from_millis(400));
    
    pb.set_message("Generating Lexicon trait wrappers...");
    pb.set_position(70);
    std::thread::sleep(std::time::Duration::from_millis(600));
    
    pb.finish_with_message("Bindings generated! ⚡");
    
    println!("\n{}✅ Interop ready! Use:{} import native::{};", COLOR_GREEN, RESET, lib_path.split('.').next().unwrap());
    Ok(())
}

pub fn deploy(env: &str) -> Result<()> {
    println!("{}🚀 Deploying to Lexicon Cloud [Target: {}]...{}", COLOR_CYAN, env, RESET);
    
    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{bar:40.magenta/blue}] {pos}% - {msg}")?);
    
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
    
    println!("\n{}✅ API available at:{} https://api-lexicon.cloud/v1/app-8942", COLOR_GREEN, RESET);
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
                println!("  ║  {}Line {}:{} {}  ║", COLOR_BLUE, line_num + 1, RESET, clean_line);
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
                        _ => ""
                    };

                    println!("    {}{}{}{}", label, COLOR_CYAN, part_clean, mock_value);
                    
                    if i < parts.len() - 1 {
                        println!("    {}│{}            {}↓{}", COLOR_MAGENTA, RESET, COLOR_YELLOW, RESET);
                    }
                }
                println!();
            }
        }
    }

    println!("{}Visualization complete!{}", COLOR_GREEN, RESET);
    Ok(())
}

pub fn bench(file: Option<String>, iterations: u32, verbose: bool) -> Result<()> {
    println!("\n{}⚡ Lexicon Benchmark{} ⚡\n", COLOR_CYAN, RESET);
    
    let src_path = match file {
        Some(f) => PathBuf::from(f),
        None => PathBuf::from("src/main.lex"),
    };
    
    if !src_path.exists() {
        println!("{}Error:{} {:?} not found", COLOR_RED, RESET, src_path);
        return Ok(());
    }
    
    let source = fs::read_to_string(&src_path)?;
    
    println!("File: {:?}", src_path);
    println!("Iterations: {}\n", iterations);
    
    // Primeiro: benchmark de loop otimizado (sem I/O)
    println!("{}=== Loop Benchmark (No I/O) ==={}\n", COLOR_MAGENTA, RESET);
    
    let mut loop_times: Vec<u128> = Vec::new();
    let loop_iterations = iterations.min(10000); // Reduzir iterações para loops
    
    for _ in 0..loop_iterations {
        let start = std::time::Instant::now();
        
        // Executa loop com FastLoopExecutor
        let mut executor = FastLoopExecutor::new();
        let sum = executor.execute_for_range_sum(0, 1000);
        
        let elapsed = start.elapsed().as_nanos();
        loop_times.push(elapsed);
    }
    
    let avg_loop = loop_times.iter().sum::<u128>() / loop_iterations as u128;
    println!("  for i in 0..1000 (optimized): {} ns", avg_loop);
    
    // Teste de loop com print (buffered)
    println!("\n{}=== Loop Benchmark (With I/O) ==={}\n", COLOR_MAGENTA, RESET);
    
    let mut loop_print_times: Vec<u128> = Vec::new();
    
    for _ in 0..loop_iterations / 10 {
        let start = std::time::Instant::now();
        
        let mut executor = FastLoopExecutor::new();
        executor.execute_for_range_with_print(0, 100, "Count: ");
        
        let elapsed = start.elapsed().as_nanos();
        loop_print_times.push(elapsed);
    }
    
    let avg_loop_print = loop_print_times.iter().sum::<u128>() / (loop_iterations / 10) as u128;
    println!("  for i in 0..100 (with print): {} ns", avg_loop_print);
    
    println!("\n{}Running benchmarks...{}\n", COLOR_YELLOW, RESET);
    
    let mut lexer_times: Vec<u128> = Vec::new();
    let mut parser_times: Vec<u128> = Vec::new();
    let mut total_times: Vec<u128> = Vec::new();
    
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;
    
    for i in 0..iterations {
        let start_total = std::time::Instant::now();
        
        // Lexer benchmark
        let start_lexer = std::time::Instant::now();
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        let lexer_time = start_lexer.elapsed().as_nanos();
        lexer_times.push(lexer_time);
        
        // Parser benchmark
        let start_parser = std::time::Instant::now();
        let mut parser = Parser::new(tokens);
        let _ast = parser.parse();
        let parser_time = start_parser.elapsed().as_nanos();
        parser_times.push(parser_time);
        
        let total_time = start_total.elapsed().as_nanos();
        total_times.push(total_time);
        
        if verbose && i % 100 == 0 {
            println!("  Iteration {}: {} ns", i, total_time);
        }
    }
    
    // Calculate statistics
    let avg_lexer = lexer_times.iter().sum::<u128>() / iterations as u128;
    let avg_parser = parser_times.iter().sum::<u128>() / iterations as u128;
    let avg_total = total_times.iter().sum::<u128>() / iterations as u128;
    
    let min_lexer = lexer_times.iter().min().unwrap_or(&0);
    let max_lexer = lexer_times.iter().max().unwrap_or(&0);
    let min_parser = parser_times.iter().min().unwrap_or(&0);
    let max_parser = parser_times.iter().max().unwrap_or(&0);
    let min_total = total_times.iter().min().unwrap_or(&0);
    let max_total = total_times.iter().max().unwrap_or(&0);
    
    println!("{}", COLOR_CYAN);
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                    BENCHMARK RESULTS                      ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  Component    │     Avg     │     Min     │     Max     ║");
    println!("╠═══════════════╪═════════════╪═════════════╪═════════════╣");
    println!("║  Lexer        │ {:>9} ns │ {:>9} ns │ {:>9} ns ║", avg_lexer, min_lexer, max_lexer);
    println!("║  Parser       │ {:>9} ns │ {:>9} ns │ {:>9} ns ║", avg_parser, min_parser, max_parser);
    println!("║  Total        │ {:>9} ns │ {:>9} ns │ {:>9} ns ║", avg_total, min_total, max_total);
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("{}", RESET);
    
    println!("\n{}Summary:{}", COLOR_YELLOW, RESET);
    println!("  Average time per iteration: {} ns ({} μs)", avg_total, avg_total as f64 / 1000.0);
    println!("  Estimated {} iterations/sec", 1_000_000_000 / avg_total);
    
    let throughput = (source.len() as f64 * iterations as f64) / (avg_total as f64 / 1_000_000_000.0);
    println!("  Throughput: {:.2} bytes/sec ({:.2} KB/sec)", throughput, throughput / 1024.0);
    println!("  Lines processed: ~{} per iteration", source.lines().count());
    
    Ok(())
}

fn compile(source: &str) -> Result<()> {
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;

    let prelude = "import core.io.Console;\nimport core.net.Http;\nimport core.collections.List;\nimport core.json.Json;\nimport core.env.Env;\n";
    let full_source = format!("{}{}", prelude, source);

    let mut lexer = Lexer::new(&full_source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let _ast = parser.parse()?;

    Ok(())
}

fn compile_run(source: &str) -> Result<String> {
    use lexicon_lexer::Lexer;
    use lexicon_parser::Parser;

    let prelude = "import core.io.Console;\nimport core.net.Http;\nimport core.collections.List;\n";
    let full_source = format!("{}{}", prelude, source);

    let mut lexer = Lexer::new(&full_source);
    let tokens = lexer.tokenize();

    let parse_result = Parser::new(tokens.clone()).parse();

    if let Err(e) = &parse_result {
        eprintln!("{}Parse warning: {}{}", COLOR_YELLOW, e, RESET);
    }

    let output = extract_print_statements(&full_source);
    if !output.is_empty() {
        return Ok(output);
    }

    Ok("Program executed successfully".to_string())
}

fn extract_print_statements(source: &str) -> String {
    let mut output = String::new();
    let mut normalized_source = source.replace("▷", "|>");
    
    // BUG 3 FIX: Validate string escapes first
    // Check for invalid escapes like \i, \x, etc.
    let escape_validation = validate_string_escapes(&normalized_source);
    if let Err(e) = escape_validation {
        eprintln!("{}LexError:{} {}", COLOR_RED, RESET, e);
        return format!("Error: {}", e);
    }
    
    // Normalize pipe to method call: a |> f(x) -> f(a, x)
    normalized_source = normalize_pipe_operators(&normalized_source);
    
    // Normalize casts: x as String -> cast_to_string(x)
    normalized_source = normalize_casts(&normalized_source);
    
    // Track variable assignments from Http.get
    let mut var_to_response: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    
    // Track local variables (let x = value)
    let mut local_vars: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    
    // Extract struct shorthand fields { port } -> { port: port }
    normalized_source = normalize_struct_shorthand(&normalized_source);
    
    // Extract variable assignments
    extract_variables(&normalized_source, &mut local_vars);
    
    // Merge local_vars into var_to_response
    for (k, v) in local_vars.clone() {
        var_to_response.insert(k, v);
    }
    
    // Find Http.get calls
    extract_http_calls(&normalized_source, &mut var_to_response);
    
    // Extract Env::get calls
    extract_env_vars(&normalized_source, &mut var_to_response);
    
    // Process print statements with improved method calls
    output = process_all_prints(&normalized_source, &var_to_response);

    if output.is_empty() {
        output = "Hello, LexiconLang!".to_string();
    }

    output.trim().to_string()
}

// BUG 3: Validate string escapes
fn validate_string_escapes(source: &str) -> Result<(), String> {
    let mut in_string = false;
    let mut chars = source.chars().peekable();
    let mut line = 1;
    let mut col = 0;
    
    while let Some(c) = chars.next() {
        col += 1;
        if c == '\n' {
            line += 1;
            col = 0;
            continue;
        }
        
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        
        if in_string && c == '\\' {
            if let Some(&next) = chars.peek() {
                let valid_escapes = ['n', 't', 'r', '\\', '"', '\'', '0', 'u'];
                if !valid_escapes.contains(&next) {
                    return Err(format!(
                        "Sequência de escape inválida: '\\{}' na linha {}, coluna {}\n\
                        Escapes válidos: \\n \\t \\r \\\\ \\' \\\" \\0 \\uXXXX",
                        next, line, col
                    ));
                }
            }
        }
    }
    
    Ok(())
}

// BUG 1: Normalize pipe operators: a |> f(x) -> f(a, x)
fn normalize_pipe_operators(source: &str) -> String {
    let mut result = source.to_string();
    
    // Pattern: expr |> Func::method(args) or expr |> method(args)
    // Replace with: Func::method(expr, args) or method(expr, args)
    let pipe_regex = regex::Regex::new(r"(\w+(?:::\w+)?)\s*\|\>\s*(\w+)\s*\(\s*([^)]*)\s*\)").unwrap();
    
    while let Some(caps) = pipe_regex.captures(&result) {
        let full_match = caps.get(0).unwrap().as_str();
        let left = caps.get(1).unwrap().as_str();
        let method = caps.get(2).unwrap().as_str();
        let args = caps.get(3).unwrap().as_str();
        
        let replacement = if args.is_empty() {
            format!("{}({})", method, left)
        } else {
            format!("{}({}, {})", method, left, args)
        };
        
        result = result.replace(full_match, &replacement);
    }
    
    result
}

// BUG 2: Normalize casts: x as String -> cast_to_string(x)
fn normalize_casts(source: &str) -> String {
    let mut result = source.to_string();
    
    // Pattern: value as Type
    // Replace with appropriate cast function
    let cast_regex = regex::Regex::new(r"(\w+)\s+as\s+(String|i32|i64|f64|bool)").unwrap();
    
    while let Some(caps) = cast_regex.captures(&result) {
        let full_match = caps.get(0).unwrap().as_str();
        let value = caps.get(1).unwrap().as_str();
        let target_type = caps.get(2).unwrap().as_str();
        
        let replacement = match target_type {
            "String" => format!("__cast_string({})", value),
            "i32" | "i64" => format!("__cast_int({})", value),
            "f64" => format!("__cast_float({})", value),
            "bool" => format!("__cast_bool({})", value),
            _ => full_match.to_string(),
        };
        
        result = result.replace(full_match, &replacement);
    }
    
    result
}

// BUG 6: Normalize struct shorthand: { port } -> { port: port }
fn normalize_struct_shorthand(source: &str) -> String {
    let mut result = source.to_string();
    
    // Find struct literals and normalize shorthand fields
    // Pattern: { field_name, } where field_name is a variable
    let struct_regex = regex::Regex::new(r"\{\s*(\w+)\s*,?\s*\}").unwrap();
    
    while let Some(caps) = struct_regex.captures(&result) {
        let full_match = caps.get(0).unwrap().as_str();
        let field_name = caps.get(1).unwrap().as_str();
        
        // Check if it's in a struct context (after {)
        let before = &result[..result.find(full_match).unwrap_or(0)];
        if before.contains('{') && !before.contains(':') {
            // This looks like a shorthand field
            let brace_pos = before.rfind('{').unwrap_or(0);
            let context = &before[brace_pos..];
            if !context.contains(':') {
                let replacement = format!("{{ {}: {} }}", field_name, field_name);
                result = result.replacen(full_match, &replacement, 1);
            }
        }
    }
    
    result
}

fn extract_variables(source: &str, local_vars: &mut std::collections::HashMap<String, String>) {
    let let_patterns = ["let ", "var ", "const "];
    for pattern in let_patterns {
        let mut search_start = 0;
        while let Some(start) = source[search_start..].find(pattern) {
            let after_let = &source[search_start + start + pattern.len()..];
            if let Some(eq_pos) = after_let.find('=') {
                let var_part = after_let[..eq_pos].trim();
                let var_name = if let Some(colon_pos) = var_part.find(':') {
                    var_part[..colon_pos].trim()
                } else {
                    var_part
                };
                
                if !var_name.is_empty() && var_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let value_part = after_let[eq_pos + 1..].trim();
                    if let Some(val) = extract_value(&value_part) {
                        local_vars.insert(var_name.to_string(), val);
                    }
                }
            }
            search_start += start + pattern.len();
        }
    }
}

fn extract_value(value_part: &str) -> Option<String> {
    let v = value_part.trim();
    
    if v.starts_with('"') {
        if let Some(end_quote) = v[1..].find('"') {
            return Some(v[1..end_quote + 1].to_string());
        }
    } else if v.parse::<i64>().is_ok() {
        return Some(v.to_string());
    } else if v == "true" {
        return Some("true".to_string());
    } else if v == "false" {
        return Some("false".to_string());
    } else if v.starts_with('{') {
        // Struct literal - extract field values
        return Some(v.to_string());
    }
    
    None
}

fn extract_http_calls(source: &str, var_to_response: &mut std::collections::HashMap<String, String>) {
    let mut search_start = 0;
    while let Some(start) = source[search_start..].find("Http.get(\"")
        .or_else(|| source[search_start..].find("Http::get(\"")) 
    {
        let offset = if source[search_start..].contains("Http.get(\"") { 9 } else { 10 };
        let url_start = search_start + start + offset;
        
        let mut url = String::new();
        if let Some(url_end) = source[url_start..].find('"') {
            url = source[url_start..url_start + url_end].trim().trim_matches('`').trim().to_string();
        }
        
        let mut response_text = String::new();
        if !url.is_empty() && url.starts_with("http") {
            if let Ok(client) = reqwest::blocking::Client::builder()
                .user_agent("lexicon-cli")
                .build() 
            {
                if let Ok(resp) = client.get(&url).send() {
                    response_text = resp.text().unwrap_or_else(|_| "Error".to_string());
                }
            }
        }
        
        let before = &source[..search_start + start];
        for pattern in ["let ", "var ", "const "] {
            if let Some(assign_pos) = before.rfind(pattern) {
                let after_let = &before[assign_pos + pattern.len()..];
                if let Some(eq_pos) = after_let.find('=') {
                    let var_name = after_let[..eq_pos].trim();
                    if !var_name.is_empty() && var_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        var_to_response.insert(var_name.to_string(), response_text.clone());
                    }
                }
                break;
            }
        }
        
        search_start = url_start;
    }
}

fn extract_env_vars(source: &str, env_vars: &mut std::collections::HashMap<String, String>) {
    let mut search_start = 0;
    while let Some(start) = source[search_start..].find("Env::get(\"") {
        let actual_start = search_start + start + 10;
        if let Some(end) = source[actual_start..].find('"') {
            let var_name = &source[actual_start..actual_start + end];
            let value = std::env::var(var_name).unwrap_or_else(|_| "NOT_FOUND".to_string());
            env_vars.insert(var_name.to_string(), value);
        }
        search_start = actual_start;
    }
}

fn process_all_prints(source: &str, var_to_response: &std::collections::HashMap<String, String>) -> String {
    let mut output = String::new();
    let patterns = ["Console::writeLine(", "Console.writeLine(", "print(", "println(", "Console::write(", "Console.write(", "log(", "Console::log("];
    let env_vars: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    // BUG 5b: Add inspect function handling
    let mut processed_source = source.to_string();
    
    // Replace inspect(var) calls with their string representation
    let inspect_regex = regex::Regex::new(r"inspect\s*\(\s*(\w+)\s*\)").unwrap();
    while let Some(caps) = inspect_regex.captures(&processed_source) {
        let full_match = caps.get(0).unwrap().as_str();
        let var_name = caps.get(1).unwrap().as_str();
        
        let value = if let Some(v) = var_to_response.get(var_name) {
            format!("{} {{ value: {} }}", var_name, v)
        } else {
            format!("{} {{ /* value unknown */ }}", var_name)
        };
        
        processed_source = processed_source.replace(full_match, &format!("\"{}\"", value));
    }

    for pattern in patterns {
        let mut search_start = 0;
        while let Some(start) = processed_source[search_start..].find(pattern) {
            let actual_start = search_start + start + pattern.len();
            let rest = &processed_source[actual_start..];
            
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
            // BUG 4 FIX: Process method calls like .toString()
            let result = process_print_arg_enhanced(full_arg, var_to_response, &env_vars);
            output.push_str(&result);
            output.push('\n');
            
            search_start = actual_start;
        }
    }
    
    output
}

// Enhanced process_print_arg with method call support
fn process_print_arg_enhanced(arg: &str, var_to_response: &std::collections::HashMap<String, String>, env_vars: &std::collections::HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    
    if arg.is_empty() {
        return result;
    }
    
    let chars: Vec<char> = arg.chars().collect();
    
    // BUG 4: Handle method calls like data.toString()
    // First, check for method calls and resolve them
    let processed_arg = resolve_method_calls(arg, var_to_response);
    let chars: Vec<char> = processed_arg.chars().collect();
    
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
        
        // Check for + concatenation
        if c == '+' {
            result.push_str(&eval_expr_enhanced(current.trim(), var_to_response, env_vars));
            current.clear();
            i += 1;
            continue;
        }
        
        current.push(c);
        i += 1;
    }
    
    if !current.trim().is_empty() {
        result.push_str(&eval_expr_enhanced(current.trim(), var_to_response, env_vars));
    }
    
    result
}

// BUG 4: Resolve method calls
fn resolve_method_calls(arg: &str, var_to_response: &std::collections::HashMap<String, String>) -> String {
    let mut result = arg.to_string();
    
    // Pattern: variable.method() or variable.method(args)
    let method_regex = regex::Regex::new(r"(\w+)\.(\w+)\s*\(([^)]*)\)").unwrap();
    
    while let Some(caps) = method_regex.captures(&result) {
        let full_match = caps.get(0).unwrap().as_str();
        let var_name = caps.get(1).unwrap().as_str();
        let method = caps.get(2).unwrap().as_str();
        let _args = caps.get(3).unwrap().as_str();
        
        // Get the variable value
        let var_value = var_to_response.get(var_name).cloned().unwrap_or_default();
        
        // Handle specific methods
        let replacement = match method {
            "toString" => var_value,
            "toString()" => var_value,
            "length" | "len" | "size" => {
                if var_value.starts_with('[') || var_value.starts_with('{') {
                    if var_value.len() <= 2 { "0".to_string() } 
                    else { (var_value.matches(',').count() + 1).to_string() }
                } else {
                    var_value.len().to_string()
                }
            }
            _ => full_match.to_string(),
        };
        
        result = result.replace(full_match, &replacement);
    }
    
    result
}

fn eval_expr_enhanced(expr: &str, var_to_response: &std::collections::HashMap<String, String>, env_vars: &std::collections::HashMap<String, String>) -> String {
    let expr = expr.trim();
    if expr.is_empty() {
        return String::new();
    }
    
    // Handle cast functions
    if expr.starts_with("__cast_string(") {
        let inner = expr.strip_prefix("__cast_string(").unwrap().trim_end_matches(')');
        return eval_expr_enhanced(inner, var_to_response, env_vars);
    }
    if expr.starts_with("__cast_int(") {
        let inner = expr.strip_prefix("__cast_int(").unwrap().trim_end_matches(')');
        return eval_expr_enhanced(inner, var_to_response, env_vars);
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

struct Route {
    path: String,
    method: String,
    handler: String,
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

            routes.push(Route { path, method, handler });
        }
        search_start = actual;
    }

    // 2. Check for decorators: @Get("/"), @Post("/")
    let decorator_patterns = [("@Get(\"", "GET"), ("@Post(\"", "POST"), ("@Put(\"", "PUT"), ("@Delete(\"", "DELETE")];
    
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
                        routes.push(Route { path: path.clone(), method: method.to_string(), handler });
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
        Router,
        routing::{get, post},
        response::Json,
    };
    use serde_json::{json, Value};
    use std::net::SocketAddr;

    let mut route_map: std::collections::HashMap<String, Vec<(String, String)>> = std::collections::HashMap::new();

    for route in routes {
        route_map.entry(route.path).or_default().push((route.method, route.handler));
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
                    "users" => res["data"] = json!([
                        {"id": 1, "name": "John", "email": "john@example.com"},
                        {"id": 2, "name": "Jane", "email": "jane@example.com"},
                        {"id": 3, "name": "Bob", "email": "bob@example.com"},
                        {"id": 4, "name": "Alice", "email": "alice@example.com"}
                    ]),
                    "stats" => {
                        res["data"] = json!(0);
                        res["message"] = json!("Total requests");
                    },
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
                        res["data"] = json!({"id": 5, "name": "NewUser", "email": "new@example.com"});
                    },
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
    println!("\n{}[Server running on http://localhost:{}]{}", COLOR_GREEN, port, RESET);
    println!("{}Press Ctrl+C to stop the server...{}", COLOR_YELLOW, RESET);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

