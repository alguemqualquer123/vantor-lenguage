// sort_numbers.lex — native sorts and searches.
// Run: lex run examples/sort_numbers.lex
import std::sort;

pub fn main() -> void {
    let xs = [9, 3, 7, 1, 8, 2, 6, 4, 5, 0];
    xs = sort::Ints(xs);
    Console::log(xs);
    Console::log("7 at: " + sort::SearchInts(xs, 7).to_string());
    let names = ["pear", "apple", "fig"];
    Console::log(sort::Strings(names));
}
