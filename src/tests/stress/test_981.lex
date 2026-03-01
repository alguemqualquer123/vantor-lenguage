module tests.test_981;

fn test_981() {
    let name = "Lexicon_981"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_981!")
}


pub fn main() {
    test_981()
}
