module tests.test_446;

fn test_446() {
    let name = "Lexicon_446"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_446!")
}


pub fn main() {
    test_446()
}
