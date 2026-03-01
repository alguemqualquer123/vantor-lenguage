module tests.test_580;

fn test_580() {
    let name = "Lexicon_580"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_580!")
}


pub fn main() {
    test_580()
}
