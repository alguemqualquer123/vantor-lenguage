// sha256sum.lex — digest lines with std::crypto::sha256.
// Run: lex run examples/sha256sum.lex
import std::crypto::sha256;

pub fn main() -> void {
    let words = ["", "abc", "hello"];
    let i = 0;
    while i < words.len() {
        Console::log(sha256::Sum(words[i]) + "  \"" + words[i] + "\"");
        i = i + 1;
    }
}
