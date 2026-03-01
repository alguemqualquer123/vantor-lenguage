module tests.test_976;

fn test_976() {
    let name = "Lexicon_976"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_976!")
}


pub fn main() {
    test_976()
}
