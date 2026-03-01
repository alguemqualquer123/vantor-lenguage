module tests.test_918;

fn test_918() {
    let name = "Lexicon_918"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_918!")
}


pub fn main() {
    test_918()
}
