module tests.test_762;

fn test_762() {
    let name = "Lexicon_762"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_762!")
}


pub fn main() {
    test_762()
}
