module tests.test_845;

fn test_845() {
    let name = "Lexicon_845"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_845!")
}


pub fn main() {
    test_845()
}
