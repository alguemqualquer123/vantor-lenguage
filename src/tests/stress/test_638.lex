module tests.test_638;

fn test_638() {
    let name = "Lexicon_638"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_638!")
}


pub fn main() {
    test_638()
}
