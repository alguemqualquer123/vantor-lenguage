module tests.test_470;

fn test_470() {
    let name = "Lexicon_470"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_470!")
}


pub fn main() {
    test_470()
}
