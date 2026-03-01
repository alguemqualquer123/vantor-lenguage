module tests.test_899;

fn test_899() {
    let name = "Lexicon_899"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_899!")
}


pub fn main() {
    test_899()
}
