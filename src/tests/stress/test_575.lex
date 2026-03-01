module tests.test_575;

fn test_575() {
    let name = "Lexicon_575"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_575!")
}


pub fn main() {
    test_575()
}
