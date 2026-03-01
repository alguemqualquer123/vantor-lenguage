module tests.test_750;

fn test_750() {
    let name = "Lexicon_750"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_750!")
}


pub fn main() {
    test_750()
}
