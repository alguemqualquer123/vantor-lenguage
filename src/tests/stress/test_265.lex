module tests.test_265;

fn test_265() {
    let name = "Lexicon_265"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_265!")
}


pub fn main() {
    test_265()
}
