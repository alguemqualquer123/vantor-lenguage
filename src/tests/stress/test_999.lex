module tests.test_999;

fn test_999() {
    let name = "Lexicon_999"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_999!")
}


pub fn main() {
    test_999()
}
