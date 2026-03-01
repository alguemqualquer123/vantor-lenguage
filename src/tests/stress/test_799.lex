module tests.test_799;

fn test_799() {
    let name = "Lexicon_799"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_799!")
}


pub fn main() {
    test_799()
}
