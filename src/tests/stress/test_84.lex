module tests.test_84;

fn test_84() {
    let name = "Lexicon_84"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_84!")
}


pub fn main() {
    test_84()
}
