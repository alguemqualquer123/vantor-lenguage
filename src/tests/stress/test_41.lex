module tests.test_41;

fn test_41() {
    let name = "Lexicon_41"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_41!")
}


pub fn main() {
    test_41()
}
