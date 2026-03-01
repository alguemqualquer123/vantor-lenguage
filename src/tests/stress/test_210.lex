module tests.test_210;

fn test_210() {
    let name = "Lexicon_210"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_210!")
}


pub fn main() {
    test_210()
}
