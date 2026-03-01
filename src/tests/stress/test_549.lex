module tests.test_549;

fn test_549() {
    let name = "Lexicon_549"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_549!")
}


pub fn main() {
    test_549()
}
