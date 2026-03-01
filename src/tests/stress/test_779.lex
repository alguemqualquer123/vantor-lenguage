module tests.test_779;

fn test_779() {
    let name = "Lexicon_779"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_779!")
}


pub fn main() {
    test_779()
}
