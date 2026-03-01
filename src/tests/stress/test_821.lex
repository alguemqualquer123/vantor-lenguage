module tests.test_821;

fn test_821() {
    let name = "Lexicon_821"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_821!")
}


pub fn main() {
    test_821()
}
