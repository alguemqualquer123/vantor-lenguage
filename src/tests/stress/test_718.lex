module tests.test_718;

fn test_718() {
    let name = "Lexicon_718"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_718!")
}


pub fn main() {
    test_718()
}
