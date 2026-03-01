module tests.test_555;

fn test_555() {
    let name = "Lexicon_555"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_555!")
}


pub fn main() {
    test_555()
}
