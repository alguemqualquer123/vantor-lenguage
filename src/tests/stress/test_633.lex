module tests.test_633;

fn test_633() {
    let name = "Lexicon_633"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_633!")
}


pub fn main() {
    test_633()
}
