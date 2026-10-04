// cli_args.lex — flags and arguments with std::flag.
// Run: lex run examples/cli_args.lex -- --name Ada --count 3
import std::flag;

pub fn main() -> void {
    flag::String("name", "world", "who to greet");
    flag::Int("count", 1, "how many times");
    flag::Bool("loud", false, "upper-case the greeting");
    flag::Parse();
    let greeting = "Hello, " + flag::GetString("name") + "!";
    if flag::GetBool("loud") {
        greeting = greeting.to_upper();
    }
    let i = 0;
    while i < flag::GetInt("count") {
        Console::log(greeting);
        i = i + 1;
    }
}
