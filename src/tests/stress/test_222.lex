module tests.test_222;

fn test_222() {
    let name = "Lexicon_222"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_222!")
}


pub fn main() {
    test_222()
}
