module tests.test_28;

fn test_28() {
    let name = "Lexicon_28"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_28!")
}


pub fn main() {
    test_28()
}
