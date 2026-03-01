module tests.test_298;

fn test_298() {
    let name = "Lexicon_298"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_298!")
}


pub fn main() {
    test_298()
}
