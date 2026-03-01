module tests.test_501;

fn test_501() {
    let name = "Lexicon_501"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_501!")
}


pub fn main() {
    test_501()
}
