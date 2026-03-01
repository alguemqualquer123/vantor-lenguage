use anyhow::Result;
use clap::Command;

mod compiler;
mod repl;
mod installer;

const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_RESET: &str = "\x1b[0m";

fn get_style() -> Command {
    Command::new("lex")
        .color(clap::ColorChoice::Always)
        .version("0.1.0-alpha")
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
            Command::new("check")
                .about("Run type checking without building ")
                .arg(clap::Arg::new("file").help("The file to check ").required(false)),
        )
        .subcommand(
            Command::new("new")
                .about("Create a new project ")
                .arg(clap::Arg::new("name").required(true).help("Project name "))
                .arg(
                    clap::Arg::new("template")
                        .long("template")
                        .short('t')
                        .help("Project template (api, plugin, service) ")
                )
                .arg(
                    clap::Arg::new("edge")
                        .long("edge")
                        .action(clap::ArgAction::SetTrue)
                        .help("Optimize for Edge Cloud (use with api template) ")
                )
                .arg(
                    clap::Arg::new("target")
                        .long("target")
                        .help("Target architecture (e.g. wasm) ")
                )
                .arg(
                    clap::Arg::new("grpc")
                        .long("grpc")
                        .action(clap::ArgAction::SetTrue)
                        .help("Enable gRPC support (use with service template) ")
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
}

fn main() -> Result<()> {
    // Tenta instalar o binário no PATH do sistema automaticamente
    let _ = installer::auto_install();

    let cli = get_style().get_matches();

    match cli.subcommand() {
        Some(("build", args)) => {
            let file = args.get_one::<String>("file").cloned();
            let release = args.get_flag("release");
            let target = args.get_one::<String>("target").cloned();
            println!("Building (release: {}, target: {:?})", release, target);
            compiler::build(file, release)?;
        }
        Some(("run", args)) => {
            let file = args.get_one::<String>("file").cloned();
            let watch = args.get_flag("watch");
            let args: Vec<String> = args
                .get_many::<String>("args")
                .map(|v| v.cloned().collect())
                .unwrap_or_default();
            
            if watch {
                compiler::watch(file, args)?;
            } else {
                compiler::run(file, args)?;
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
            println!("Running benchmark (iterations: {}, verbose: {})", iterations, verbose);
            compiler::bench(file, iterations, verbose)?;
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
        None => {
            println!("{}🔮 Lexicon{} Compiler v0.1.0-alpha", COLOR_CYAN, COLOR_RESET);
            println!("\nUsage: lex <command>");
            println!("\nRun 'lex --help' for more information.");
        }
        _ => {}
    }

    Ok(())
}
