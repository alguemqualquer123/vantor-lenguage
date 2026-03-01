module tests.test_551;

fn test_551() {
    let name = "Lexicon_551"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_551!")
}


pub fn main() {
    test_551()
}
