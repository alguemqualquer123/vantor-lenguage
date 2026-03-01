module tests.test_741;

fn test_741() {
    let name = "Lexicon_741"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_741!")
}


pub fn main() {
    test_741()
}
