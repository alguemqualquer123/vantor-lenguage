use lexicon_analysis::typeck::TypeChecker;
use lexicon_lexer::Lexer;
use lexicon_parser::Parser;
use std::io::{self, Write};
use std::collections::HashMap;

pub struct Repl {
    history: Vec<String>,
    variables: HashMap<String, String>,
    line_number: usize,
}

impl Repl {
    pub fn new() -> Self {
        Repl {
            history: Vec::new(),
            variables: HashMap::new(),
            line_number: 1,
        }
    }

    pub fn run(&mut self) {
        println!("Lexicon REPL 1.0.0");
        println!("Type :help for commands, :quit to exit\n");

        loop {
            print!("lexicon> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).unwrap() == 0 {
                break;
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            // Add to history
            self.history.push(input.to_string());

            // Handle special commands
            if input.starts_with(':') {
                self.handle_command(input);
                continue;
            }

            // Try to compile and run
            match self.eval(input) {
                Ok(output) => {
                    if !output.is_empty() {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }

            self.line_number += 1;
        }

        println!("\nGoodbye!");
    }

    fn handle_command(&mut self, cmd: &str) {
        match cmd {
            ":help" => {
                println!("Lexicon REPL Commands:");
                println!("  :help       - Show this help");
                println!("  :quit       - Exit REPL");
                println!("  :clear      - Clear screen");
                println!("  :history    - Show command history");
                println!("  :vars       - Show variables");
                println!("  :type <expr> - Show type of expression");
                println!("  :run <code> - Run Lexicon code");
                println!("  :ast <code> - Show AST of expression");
            }
            ":quit" | ":q" => {
                std::process::exit(0);
            }
            ":clear" | ":cls" => {
                print!("\x1B[2J\x1B[1J");
                print!("\x1B[3J");
                io::stdout().flush().unwrap();
            }
            ":history" | ":hist" => {
                for (i, cmd) in self.history.iter().enumerate() {
                    println!("{}: {}", i + 1, cmd);
                }
            }
            ":vars" => {
                if self.variables.is_empty() {
                    println!("No variables defined");
                } else {
                    for (name, value) in &self.variables {
                        println!("{} = {}", name, value);
                    }
                }
            }
            ":type" => {
                println!("Usage: :type <expression>");
            }
            cmd if cmd.starts_with(":type ") => {
                let expr = &cmd[6..];
                self.show_type(expr);
            }
            cmd if cmd.starts_with(":run ") => {
                let code = &cmd[5..];
                match self.eval(code) {
                    Ok(output) => println!("{}", output),
                    Err(e) => println!("Error: {}", e),
                }
            }
            cmd if cmd.starts_with(":ast ") => {
                let code = &cmd[5..];
                self.show_ast(code);
            }
            _ => {
                println!("Unknown command. Type :help for help.");
            }
        }
    }

    fn eval(&self, code: &str) -> Result<String, String> {
        // Parse the code
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| e.to_string())?;

        // Check types
        let mut type_checker = TypeChecker::new();
        if let Err(errors) = type_checker.check_module(&ast) {
            return Err(format!("Type errors: {:?}", errors));
        }

        // For now, just extract and run print statements
        let output = extract_prints(code);

        if output.is_empty() {
            // Try to evaluate as expression
            if code.contains('=') {
                Ok(format!("Compiled successfully"))
            } else {
                Ok(format!("Expression evaluated"))
            }
        } else {
            Ok(output)
        }
    }

    fn show_type(&self, expr: &str) {
        let code = format!("let _ = {};", expr);
        match self.eval(&code) {
            _ => println!("Type: unknown (type inference requires more work)"),
        }
    }

    fn show_ast(&self, code: &str) {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();

        println!("Tokens:");
        for t in &tokens {
            println!("  {:?}", t.token);
        }

        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(ast) => println!("\nAST: {:?}", ast),
            Err(e) => println!("\nParse error: {}", e),
        }
    }
}

fn extract_prints(source: &str) -> String {
    let mut output = String::new();
    let mut chars = source.chars().peekable();
    let mut in_string = false;
    let mut current_string = String::new();

    while let Some(ch) = chars.next() {
        if ch == '"' && !in_string {
            in_string = true;
            current_string.clear();
        } else if ch == '"' && in_string {
            in_string = false;
            // Check if this is inside a print function
            let before_len = source.len() - source[source.len() - 1..].len();
            let before = &source[..before_len.min(source.len())];
            if before.contains("print(") || before.contains("writeLine") || before.contains("log(")
            {
                output.push_str(&current_string);
                output.push('\n');
            }
        } else if in_string {
            if ch == '\\' {
                if let Some(&next) = chars.peek() {
                    let escaped = match next {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        _ => next,
                    };
                    current_string.push(escaped);
                    chars.next();
                }
            } else {
                current_string.push(ch);
            }
        }
    }

    output
}

pub fn start_repl() {
    let mut repl = Repl::new();
    repl.run();
}
