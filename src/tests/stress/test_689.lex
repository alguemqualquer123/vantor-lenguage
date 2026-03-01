module tests.test_689;

fn test_689() {
    let name = "Lexicon_689"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_689!")
}


pub fn main() {
    test_689()
}
