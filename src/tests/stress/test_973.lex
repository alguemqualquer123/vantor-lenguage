module tests.test_973;

fn test_973() {
    let name = "Lexicon_973"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_973!")
}


pub fn main() {
    test_973()
}
