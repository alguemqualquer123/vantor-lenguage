// json_pretty.lex — parse, validate and re-emit JSON.
// Run: lex run examples/json_pretty.lex
import std::encoding::json;

pub fn main() -> void {
    let raw = "[{\"name\":\"Ada\",\"year\":1843},{\"name\":\"Grace\",\"year\":1906}]";
    if !json::Valid(raw) {
        Console::log("invalid JSON");
        return;
    }
    let people = json::Unmarshal(raw);
    Console::log("first: " + people[0].name);
    Console::log(json::Marshal(people));
}
