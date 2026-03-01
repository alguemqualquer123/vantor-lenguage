module tests.test_589;

fn test_589() {
    let name = "Lexicon_589"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_589!")
}


pub fn main() {
    test_589()
}
