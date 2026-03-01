module tests.test_6;

fn test_6() {
    let name = "Lexicon_6"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_6!")
}


pub fn main() {
    test_6()
}
