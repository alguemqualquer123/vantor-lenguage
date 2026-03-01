module tests.test_791;

fn test_791() {
    let name = "Lexicon_791"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_791!")
}


pub fn main() {
    test_791()
}
