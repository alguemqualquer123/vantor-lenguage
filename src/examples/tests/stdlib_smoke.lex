// Smoke test — Fase 0: module loader + Fase 1: std::strings (Go parity).
import std::strings;

fn main() {
    Console::log(strings::ToUpper("hello lexicon"));
    Console::log(strings::HasPrefix("hello world", "hello"));
    Console::log(strings::Join(["a", "b", "c"], "-"));
    Console::log(strings::Repeat("ab", 3));
    Console::log(strings::Index("hello", "ll"));
    Console::log(strings::TrimSpace("  padded  "));
    Console::log(strings::ReplaceAll("a-b-c", "-", "+"));
    Console::log(strings::Split("x,y,z", ","));
    Console::log(strings::RuneCount("lexicon"));
}
