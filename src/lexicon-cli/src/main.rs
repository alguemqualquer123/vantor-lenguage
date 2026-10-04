use anyhow::Result;
use clap::Command;
use env_logger::Env;
use log::{debug, info};

mod compiler;
mod interp;
mod commands;
mod repl;
mod installer;
mod sdk;
mod complete;
mod lsp;
mod dap;
mod ide;
#[cfg(feature = "gui")]
mod gui;

const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_RESET: &str = "\x1b[0m";

fn get_style() -> Command {
    Command::new("lex")
        .color(clap::ColorChoice::Always)
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(false)
        .arg_required_else_help(false)
        .about(format!(
            "{}🔮 Lexicon{} Compiler",
            COLOR_CYAN, COLOR_RESET
        ))
        .long_about(format!(
            r#"{}🔮 Lexicon Compiler {}

A modern compiler for the Lexicon programming language.

Available Commands:
  build       Compile the project to an executable
  run         Compile and run the main program
  test        Run the test suite
  bench       Run benchmark tests
  check       Run type checking without building
  fmt         Format the source code
  lint        Lint source (unused, dead code, suspicious APIs)
  vet         Static correctness checks
  doc         Generate API docs
  trace       Capture execution traces
  profile     Profile compilation stages
  debug       Launch debugger
  generate    Run code generation
  mod         Manage modules/dependencies
  env         Inspect toolchain environment
  version     Show versions
  clean       Remove build artifacts
  publish     Publish package
  repl        Start the interactive REPL
  new         Create a new project
  init        Initialize a new project in current directory
  install     Install Lexicon globally in the system PATH
  uninstall   Uninstall Lexicon from the system
  visualize   Visualize data flow in pipes
  deploy      Deploy your Lexicon API to the Cloud
  ffi         Generate bindings for C/Rust interop

Quick Start:
  lex new myproject           Create a new project
  cd myproject                Enter project directory
  lex run                     Build and run
  lex bench                   Run benchmark
  lex visualize src/main.lex  See pipes in action"#,
            COLOR_CYAN, COLOR_RESET
        ))
        .subcommand(
            Command::new("visualize")
                .about("Visualize data flow in pipes ")
                .arg(clap::Arg::new("file").required(true).help("File to visualize ")),
        )
        .subcommand(
            Command::new("build")
                .about("Compile the project to an executable ")
                .arg(clap::Arg::new("file").help("The file to build ").required(false))
                .arg_required_else_help(false)
                .arg(
                    clap::Arg::new("release")
                        .short('r')
                        .long("release")
                        .action(clap::ArgAction::SetTrue)
                        .help("Build in release mode "),
                )
                .arg(
                    clap::Arg::new("target")
                        .short('t')
                        .long("target")
                        .help("Target architecture (e.g. wasm, x86_64, aarch64) "),
                )
                .arg(
                    clap::Arg::new("features")
                        .short('F')
                        .long("features")
                        .help("Comma-separated feature flags for #[cfg(feature = \"..\")] "),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("Compile and run a Lexicon program ")
                .arg(clap::Arg::new("file").help("The file to run ").required(false))
                .arg(
                    clap::Arg::new("watch")
                        .short('w')
                        .long("watch")
                        .action(clap::ArgAction::SetTrue)
                        .help("Hot Reload: Watch for file changes and re-run automatically "),
                )
                .arg(
                    clap::Arg::new("ci")
                        .long("ci")
                        .action(clap::ArgAction::SetTrue)
                        .help("CI mode: non-interactive, never prompt (also enabled by CI=true) "),
                )
                .arg(clap::Arg::new("args").last(true).num_args(0..)),
        )
        .subcommand(Command::new("repl").about("Start the interactive REPL "))
        .subcommand(
            Command::new("test")
                .about("Run the test suite ")
                .arg(
                    clap::Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show verbose output "),
                )
                .arg(
                    clap::Arg::new("coverage")
                        .short('c')
                        .long("coverage")
                        .action(clap::ArgAction::SetTrue)
                        .help("Run with coverage "),
                ),
        )
        .subcommand(
            Command::new("bench")
                .about("Run benchmark tests ")
                .arg(clap::Arg::new("file").help("The file to benchmark ").required(false))
                .arg(
                    clap::Arg::new("iterations")
                        .short('n')
                        .long("iterations")
                        .help("Number of iterations (default: 1000) "),
                )
                .arg(
                    clap::Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show detailed output "),
                )
                .arg(
                    clap::Arg::new("ci")
                        .long("ci")
                        .action(clap::ArgAction::SetTrue)
                        .help("CI mode: non-interactive (also enabled by CI=true) "),
                ),
        )
        .subcommand(
            Command::new("fmt").about("Format the source code ").arg(
                clap::Arg::new("check")
                    .short('c')
                    .long("check")
                    .help("Check formatting without modifying "),
            ),
        )
        .subcommand(
            Command::new("lint")
                .about("Lint source (unused, dead code, suspicious, dangerous APIs) ")
                .arg(
                    clap::Arg::new("json")
                        .long("json")
                        .action(clap::ArgAction::SetTrue)
                        .help("Machine-readable JSON output "),
                ),
        )
        .subcommand(
            Command::new("vet")
                .about("Static correctness checks (type, interface, ABI) ")
                .arg(clap::Arg::new("file").help("The file to vet ").required(false)),
        )
        .subcommand(
            Command::new("doc")
                .about("Generate API docs from doc comments ")
                .arg(clap::Arg::new("file").help("The file to document ").required(false)),
        )
        .subcommand(
            Command::new("trace")
                .about("Capture execution trace (stage timings) ")
                .arg(clap::Arg::new("file").help("The file to trace ").required(false)),
        )
        .subcommand(
            Command::new("profile")
                .about("Profile compilation stages ")
                .arg(clap::Arg::new("file").help("The file to profile ").required(false)),
        )
        .subcommand(
            Command::new("debug")
                .about("Launch debugger (symbol inspection) ")
                .arg(clap::Arg::new("file").help("The file to debug ").required(false)),
        )
        .subcommand(
            Command::new("serve")
                .about("Start the real HTTP server (Node http parity) ")
                .arg(clap::Arg::new("file").help("The file to serve ").required(false))
                .arg(clap::Arg::new("host").long("host").default_value("127.0.0.1").help("Bind host "))
                .arg(clap::Arg::new("port").long("port").default_value("3000").help("Bind port ")),
        )
        .subcommand(
            Command::new("test-net")
                .about("Smoke-test TCP/HTTP/UDP/URL/WS (Node net parity) ")
                .arg(clap::Arg::new("host").long("host").default_value("127.0.0.1").help("Target host "))
                .arg(clap::Arg::new("port").long("port").default_value("3000").help("Target port ")),
        )
        .subcommand(
            Command::new("dns-check")
                .about("Resolve a host (Node dns.lookup parity) ")
                .arg(clap::Arg::new("host").long("host").default_value("example.com").help("Host or URL to resolve "))
                .arg(clap::Arg::new("timeout").long("timeout").default_value("5s").help("Timeout like 5s/500ms ")),
        )
        .subcommand(
            Command::new("tls-check")
                .about("Validate TLS config for a host (Node tls parity) ")
                .arg(clap::Arg::new("host").long("host").default_value("example.com").help("SNI host "))
                .arg(clap::Arg::new("port").long("port").default_value("443").help("TLS port ")),
        )
        .subcommand(Command::new("generate").about("Run code generation (macro sites) "))
        .subcommand(
            Command::new("mod")
                .about("Manage modules/dependencies (validate manifest) ")
                .arg(clap::Arg::new("args").num_args(0..)),
        )
        .subcommand(Command::new("env").about("Inspect toolchain environment "))
        .subcommand(Command::new("version").about("Show versions "))
        .subcommand(Command::new("clean").about("Remove build artifacts "))
        .subcommand(
            Command::new("publish")
                .about("Publish package (validates metadata) ")
                .arg(
                    clap::Arg::new("dry-run")
                        .long("dry-run")
                        .action(clap::ArgAction::SetTrue)
                        .help("Validate only "),
                ),
        )
        .subcommand(
            Command::new("benchmark")
                .about("Run benchmarks (alias of bench) ")
                .arg(clap::Arg::new("file").help("The file to benchmark ").required(false))
                .arg(
                    clap::Arg::new("ci")
                        .long("ci")
                        .action(clap::ArgAction::SetTrue)
                        .help("CI mode: non-interactive (also enabled by CI=true) "),
                ),
        )
        .subcommand(
            Command::new("check")
                .about("Run type checking without building ")
                .arg(clap::Arg::new("file").help("The file to check ").required(false)),
        )
        .subcommand(
            Command::new("fix")
                .about("Apply automatic migrations for breaking changes ")
                .arg(clap::Arg::new("file").help("The file to fix ").required(false))
                .arg(
                    clap::Arg::new("dry-run")
                        .long("dry-run")
                        .action(clap::ArgAction::SetTrue)
                        .help("Preview changes without modifying "),
                ),
        )
        .subcommand(
            Command::new("new")
                .about("Create a new project (each template generates runnable code) ")
                .arg(clap::Arg::new("name").required(true).help("Project name "))
                .arg(
                    clap::Arg::new("template")
                        .long("template")
                        .short('t')
                        .help("Project template: api (REST on :3000), plugin (WASM-ready transform), service (gRPC IDL + health gateway). Unknown names are rejected. ")
                )
                .arg(
                    clap::Arg::new("edge")
                        .long("edge")
                        .action(clap::ArgAction::SetTrue)
                        .help("Edge variant of the api template (JSON on :8080) ")
                )
                .arg(
                    clap::Arg::new("target")
                        .long("target")
                        .help("Target architecture; plugin + --target wasm marks the WASM export ")
                )
                .arg(
                    clap::Arg::new("grpc")
                        .long("grpc")
                        .action(clap::ArgAction::SetTrue)
                        .help("Wire the Grpc runtime serve call into the service template ")
                ),
        )
        .subcommand(Command::new("init").about("Initialize a new project in current directory "))
        .subcommand(Command::new("install").about("Install Lexicon globally in the system PATH "))
        .subcommand(Command::new("uninstall").about("Uninstall Lexicon from the system "))
        .subcommand(
            Command::new("deploy")
                .about("Deploy your Lexicon API to the Cloud ")
                .arg(clap::Arg::new("env").long("env").help("Environment (prod/staging) ")),
        )
        .subcommand(
            Command::new("ffi")
                .about("Native Interop: Generate C/Rust bindings ")
                .arg(clap::Arg::new("lib").required(true).help("Library path ")),
        )
        .subcommand(
            Command::new("gui")
                .about("Run a Lexicon GUI application (native)")
                .arg(clap::Arg::new("file").help("The GUI file to run ").required(false)),
        )
        .subcommand(
            Command::new("sdk")
                .about("Package/verify the Lexicon SDK kit ")
                .arg(
                    clap::Arg::new("action")
                        .help("export (default), verify, info ")
                        .required(false),
                )
                .arg(
                    clap::Arg::new("out")
                        .long("out")
                        .help("Export/verify directory (default: build/sdk) "),
                ),
        )
        .subcommand(
            Command::new("complete")
                .about("Autocomplete: suggest modules/members/keywords ")
                .arg(
                    clap::Arg::new("prefix")
                        .long("prefix")
                        .help("Prefix to complete (e.g. \"Http::\") "),
                )
                .arg(clap::Arg::new("file").long("file").help("Source file "))
                .arg(clap::Arg::new("line").long("line").help("1-based line "))
                .arg(clap::Arg::new("col").long("col").help("0-based column "))
                .arg(
                    clap::Arg::new("json")
                        .long("json")
                        .action(clap::ArgAction::SetTrue)
                        .help("Machine-readable JSON output "),
                ),
        )
        .subcommand(Command::new("lsp").about("Start the Lexicon language server (stdio) "))
        .subcommand(Command::new("dap").about("Start the Lexicon debug adapter (DAP over stdio) "))
        .subcommand(
            Command::new("ide")
                .about("Scaffold editor assets (VSCode tasks + snippets) ")
                .arg(
                    clap::Arg::new("action")
                        .help("init (default) | extension ")
                        .required(false),
                )
                .arg(clap::Arg::new("dir").help("Target directory (default: cwd) ").required(false)),
        )
    }


fn main() -> Result<()> {
    // Inicializa logger global (usa RUST_LOG ou padrão info)
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    debug!("Logger inicializado");

    // Auto-install é caro (copia binário de ~250MB + checa PATH a cada
    // invocação) e poluía o stdout dos benchmarks. Pula em modo CI
    // (CI=true ou LEXICON_NO_AUTO_INSTALL=1) — `lex install` continua manual.
    {
        let ci = std::env::var("CI")
            .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes"))
            .unwrap_or(false);
        let no_auto = std::env::var("LEXICON_NO_AUTO_INSTALL").is_ok();
        if !ci && !no_auto {
            let _ = installer::auto_install();
        } else {
            debug!("auto_install pulado (modo CI/não-interativo)");
        }
    }

    let cli = get_style().get_matches();
    debug!("CLI args parseados");

    match cli.subcommand() {
        Some(("build", args)) => {
            let file = args.get_one::<String>("file").cloned();
            let release = args.get_flag("release");
            let target = args.get_one::<String>("target").cloned();
            let features = args.get_one::<String>("features").cloned();
            info!("Building (release: {}, target: {:?}, features: {:?})", release, target, features);
            compiler::build_with_target(file, release, target, features)?;
        }
        Some(("run", args)) => {
            let file = args.get_one::<String>("file").cloned();
            let watch = args.get_flag("watch");
            let ci = args.get_flag("ci");
            let args: Vec<String> = args
                .get_many::<String>("args")
                .map(|v| v.cloned().collect())
                .unwrap_or_default();
            
            if watch {
                compiler::watch(file, args)?;
            } else {
                compiler::run_with_ci(file, args, ci)?;
            }
        }
        Some(("repl", _)) => {
            println!("Starting LexiconLang REPL...\n");
            repl::start_repl();
        }
        Some(("test", args)) => {
            let verbose = args.get_flag("verbose");
            let coverage = args.get_flag("coverage");
            println!("Testing (verbose: {}, coverage: {})", verbose, coverage);
            compiler::test(verbose)?;
        }
        Some(("bench", args)) => {
            let file = args.get_one::<String>("file").cloned();
            let iterations = args.get_one::<String>("iterations")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000);
            let verbose = args.get_flag("verbose");
            let ci = args.get_flag("ci");
            println!("Running benchmark (iterations: {}, verbose: {}, ci: {})", iterations, verbose, ci);
            compiler::bench_with_ci(file, iterations, verbose, ci)?;
        }
        Some(("benchmark", args)) => {
            let file = args.get_one::<String>("file").cloned();
            let ci = args.get_flag("ci");
            println!("Running benchmark...");
            compiler::bench_with_ci(file, 1000, false, ci)?;
        }
        Some(("lint", args)) => {
            let json = args.get_flag("json");
            compiler::lint(json)?;
        }
        Some(("vet", args)) => {
            let file = args.get_one::<String>("file").cloned();
            compiler::vet(file)?;
        }
        Some(("doc", args)) => {
            let file = args.get_one::<String>("file").cloned();
            compiler::doc_cmd(file)?;
        }
        Some(("trace", args)) => {
            let file = args.get_one::<String>("file").cloned();
            compiler::trace_cmd(file)?;
        }
        Some(("profile", args)) => {
            let file = args.get_one::<String>("file").cloned();
            compiler::profile(file)?;
        }
        Some(("debug", args)) => {
            let file = args.get_one::<String>("file").cloned();
            compiler::debug_cmd(file)?;
        }
        Some(("serve", args)) => {
            let file = args.get_one::<String>("file").cloned();
            let host = args.get_one::<String>("host").cloned().unwrap_or_else(|| "127.0.0.1".to_string());
            let port = args.get_one::<String>("port").and_then(|s| s.parse().ok()).unwrap_or(3000);
            compiler::serve_net(file, host, port)?;
        }
        Some(("test-net", args)) => {
            let host = args.get_one::<String>("host").cloned().unwrap_or_else(|| "127.0.0.1".to_string());
            let port = args.get_one::<String>("port").and_then(|s| s.parse().ok()).unwrap_or(3000);
            compiler::test_net(host, port)?;
        }
        Some(("dns-check", args)) => {
            let host = args.get_one::<String>("host").cloned().unwrap_or_else(|| "example.com".to_string());
            let timeout_raw = args.get_one::<String>("timeout").cloned().unwrap_or_else(|| "5s".to_string());
            let secs = parse_timeout_secs(&timeout_raw);
            compiler::dns_check_cmd(host, secs)?;
        }
        Some(("tls-check", args)) => {
            let host = args.get_one::<String>("host").cloned().unwrap_or_else(|| "example.com".to_string());
            let port = args.get_one::<String>("port").and_then(|s| s.parse().ok()).unwrap_or(443);
            compiler::tls_check_cmd(host, port)?;
        }
        Some(("generate", _)) => {
            compiler::generate()?;
        }
        Some(("mod", args)) => {
            let rest: Vec<String> = args
                .get_many::<String>("args")
                .map(|v| v.cloned().collect())
                .unwrap_or_default();
            compiler::mod_cmd(rest)?;
        }
        Some(("env", _)) => {
            compiler::env_cmd()?;
        }
        Some(("version", _)) => {
            compiler::version_cmd()?;
        }
        Some(("clean", _)) => {
            compiler::clean()?;
        }
        Some(("publish", args)) => {
            let dry_run = args.get_flag("dry-run");
            compiler::publish(dry_run)?;
        }
        Some(("fmt", args)) => {
            let check = args.get_flag("check");
            println!("Formatting (check: {})", check);
            compiler::fmt(check)?;
        }
        Some(("check", args)) => {
            let file = args.get_one::<String>("file").cloned();
            println!("Type checking...");
            compiler::check(file)?;
        }
        Some(("fix", args)) => {
            let file = args.get_one::<String>("file").cloned();
            let dry_run = args.get_flag("dry-run");
            compiler::fix(file, dry_run)?;
        }
        Some(("new", args)) => {
            let name = args.get_one::<String>("name").unwrap();
            let template = args.get_one::<String>("template").cloned();
            let edge = args.get_flag("edge");
            let target = args.get_one::<String>("target").cloned();
            let grpc = args.get_flag("grpc");
            compiler::new_project(name, template, edge, target, grpc)?;
        }
        Some(("visualize", args)) => {
            let file = args.get_one::<String>("file").unwrap();
            compiler::visualize(file)?;
        }
        Some(("init", _)) => {
            println!("Initializing project...");
        }
        Some(("install", _)) => {
            println!("Installing Lexicon globally...");
            installer::manual_install()?;
        }
        Some(("uninstall", _)) => {
            println!("Uninstalling Lexicon...");
            installer::uninstall()?;
        }
        Some(("deploy", args)) => {
            let env = args.get_one::<String>("env").map(|s| s.as_str()).unwrap_or("prod");
            compiler::deploy(env)?;
        }
        Some(("ffi", args)) => {
            let lib = args.get_one::<String>("lib").unwrap();
            compiler::ffi(lib)?;
        }
        Some(("gui", args)) => {
            let file = args.get_one::<String>("file").cloned();
            compiler::run_gui(file)?;
        }
        Some(("sdk", args)) => {
            let action = args
                .get_one::<String>("action")
                .map(|s| s.as_str())
                .unwrap_or("export");
            let out = args.get_one::<String>("out").cloned();
            match action {
                "export" => sdk::export(out)?,
                "verify" => sdk::verify(out)?,
                "info" => sdk::info()?,
                other => {
                    println!("Unknown sdk action '{}'. Use export|verify|info.", other);
                }
            }
        }
        Some(("complete", args)) => {
            let prefix = args.get_one::<String>("prefix").cloned();
            let file = args.get_one::<String>("file").cloned();
            let line = args
                .get_one::<String>("line")
                .and_then(|s| s.parse().ok());
            let col = args
                .get_one::<String>("col")
                .and_then(|s| s.parse().ok());
            let json = args.get_flag("json");
            complete::complete_cmd(prefix, file, line, col, json)?;
        }
        Some(("lsp", _)) => {
            lsp::serve()?;
        }
        Some(("dap", _)) => {
            dap::serve()?;
        }
        Some(("ide", args)) => {
            let action = args
                .get_one::<String>("action")
                .map(|s| s.as_str())
                .unwrap_or("init");
            let dir = args.get_one::<String>("dir").cloned();
            match action {
                "init" => ide::init(dir)?,
                "extension" => ide::extension(dir)?,
                other => {
                    println!("Unknown ide action '{}'. Use init|extension.", other);
                }
            }
        }
        None => {
            println!("{}🔮 Lexicon{} Compiler v{}", COLOR_CYAN, COLOR_RESET, env!("CARGO_PKG_VERSION"));
            println!("\nUsage: lex <command>");
            println!("\nRun 'lex --help' for more information.");
        }
        _ => {}
    }

    Ok(())
}

/// Parse `--timeout 5s|500ms|2m` into seconds (min 1, max 30).
/// Used by `lex dns-check --timeout`.
fn parse_timeout_secs(raw: &str) -> u64 {
    let t = raw.trim().to_ascii_lowercase();
    let num_end = t.find(|c: char| !c.is_ascii_digit()).unwrap_or(t.len());
    let n: u64 = t[..num_end].parse().unwrap_or(5);
    if t.ends_with("ms") {
        (n.div_ceil(1000)).clamp(1, 30)
    } else if t.ends_with('m') {
        (n * 60).clamp(1, 30)
    } else {
        n.clamp(1, 30)
    }
}
