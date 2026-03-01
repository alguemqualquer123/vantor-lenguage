module tests.test_875;

fn test_875() {
    let name = "Lexicon_875"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_875!")
}


pub fn main() {
    test_875()
}
