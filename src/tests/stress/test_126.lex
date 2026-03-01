module tests.test_126;

fn test_126() {
    let name = "Lexicon_126"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_126!")
}


pub fn main() {
    test_126()
}
