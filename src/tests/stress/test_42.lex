module tests.test_42;

fn test_42() {
    let name = "Lexicon_42"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_42!")
}


pub fn main() {
    test_42()
}
