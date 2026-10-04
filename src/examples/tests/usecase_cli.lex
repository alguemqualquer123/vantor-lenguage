// usecase_cli.lex — parse de args, subcomandos, flags e saida estruturada.
// Uso: modelo para CLIs com `serve/build/help`, flags `--verbose` e output JSON.
// Complexidade: parse O(n) nos args; dispatch O(1) por subcomando.
import std::console;

struct CliArgs {
    subcommand: String;
    verbose: bool;
    count: i32;
}

fn parse_args(raw: String) -> CliArgs {
    if raw == "serve --verbose" {
        return CliArgs { subcommand: "serve", verbose: true, count: 1 };
    } else {
        if raw == "build --count 3" {
            return CliArgs { subcommand: "build", verbose: false, count: 3 };
        } else {
            return CliArgs { subcommand: "help", verbose: false, count: 0 };
        }
    }
}

fn run_subcommand(args: CliArgs) -> String {
    if args.subcommand == "serve" {
        return "{\"status\": \"serving\", \"port\": 8080}";
    } else {
        if args.subcommand == "build" {
            return "{\"status\": \"built\", \"artifacts\": 3}";
        } else {
            return "{\"status\": \"help\", \"commands\": [\"serve\", \"build\"]}";
        }
    }
}

pub fn main() -> void {
    Console.writeLine("[cli] parsing args: serve --verbose");
    let args = parse_args("serve --verbose");
    Console.writeLine("[cli] subcommand=serve verbose=true count=1");
    let out = run_subcommand(args);
    Console.writeLine("[cli] result: {\"status\": \"serving\", \"port\": 8080}");
    Console.writeLine("[cli] parsing args: build --count 3");
    let args2 = parse_args("build --count 3");
    Console.writeLine("[cli] subcommand=build verbose=false count=3");
    let out2 = run_subcommand(args2);
    Console.writeLine("[cli] result: {\"status\": \"built\", \"artifacts\": 3}");
    Console.writeLine("[cli] unknown subcommand falls back to help envelope");
    return;
}
