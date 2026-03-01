module tests.test_724;

fn test_724() {
    let name = "Lexicon_724"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_724!")
}


pub fn main() {
    test_724()
}
