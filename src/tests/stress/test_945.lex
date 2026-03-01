module tests.test_945;

fn test_945() {
    let name = "Lexicon_945"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_945!")
}


pub fn main() {
    test_945()
}
