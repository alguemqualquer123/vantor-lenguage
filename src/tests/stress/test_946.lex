module tests.test_946;

fn test_946() {
    let name = "Lexicon_946"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_946!")
}


pub fn main() {
    test_946()
}
