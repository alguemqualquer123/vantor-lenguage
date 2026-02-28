use clap::{Parser, Subcommand};
use anyhow::Result;

mod compiler;
mod repl;

#[derive(Parser)]
#[command(name = "vantor")]
#[command(about = "VantorLang Compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(short, long)]
        release: bool,
        #[arg(short, long)]
        target: Option<String>,
    },
    Run {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    Repl,
    Test {
        #[arg(short, long)]
        verbose: bool,
        #[arg(short, long)]
        coverage: bool,
    },
    Fmt {
        #[arg(short, long)]
        check: bool,
    },
    Check,
    New {
        name: String,
    },
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Build { release, target } => {
            println!("Building (release: {}, target: {:?})", release, target);
            compiler::build(release)?;
        }
        Commands::Run { args } => {
            compiler::run(args)?;
        }
        Commands::Repl => {
            println!("Starting VantorLang REPL...\n");
            repl::start_repl();
        }
        Commands::Test { verbose, coverage } => {
            println!("Testing (verbose: {}, coverage: {})", verbose, coverage);
            compiler::test(verbose)?;
        }
        Commands::Fmt { check } => {
            println!("Formatting (check: {})", check);
            compiler::fmt(check)?;
        }
        Commands::Check => {
            println!("Type checking...");
            compiler::check()?;
        }
        Commands::New { name } => {
            compiler::new_project(&name)?;
        }
        Commands::Init => {
            println!("Initializing project...");
        }
    }
    
    Ok(())
}
