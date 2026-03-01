module tests.test_13;

fn test_13() {
    let name = "Lexicon_13"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_13!")
}


pub fn main() {
    test_13()
}
