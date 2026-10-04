// time_now.lex — clock, formatting and round-trip parsing.
import std::time;

pub fn main() -> void {
    let now = time::Now();
    Console::log(time::Format(now, "2006-01-02 15:04:05"));
    let back = time::ParseDate(time::Format(now, "2006-01-02"));
    Console::log(back.ok);
}
