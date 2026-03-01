module tests.test_988;

fn test_988() {
    let name = "Lexicon_988"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_988!")
}


pub fn main() {
    test_988()
}
