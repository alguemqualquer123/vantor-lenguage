module tests.test_790;

fn test_790() {
    let name = "Lexicon_790"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_790!")
}


pub fn main() {
    test_790()
}
