module tests.test_562;

fn test_562() {
    let name = "Lexicon_562"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_562!")
}


pub fn main() {
    test_562()
}
