module tests.test_171;

fn test_171() {
    let name = "Lexicon_171"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_171!")
}


pub fn main() {
    test_171()
}
