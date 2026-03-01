module tests.test_124;

fn test_124() {
    let name = "Lexicon_124"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_124!")
}


pub fn main() {
    test_124()
}
