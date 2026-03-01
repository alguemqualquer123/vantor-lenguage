module tests.test_796;

fn test_796() {
    let name = "Lexicon_796"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_796!")
}


pub fn main() {
    test_796()
}
