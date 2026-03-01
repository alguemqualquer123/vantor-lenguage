module tests.test_49;

fn test_49() {
    let name = "Lexicon_49"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_49!")
}


pub fn main() {
    test_49()
}
