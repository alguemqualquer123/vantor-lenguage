module tests.test_559;

fn test_559() {
    let name = "Lexicon_559"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_559!")
}


pub fn main() {
    test_559()
}
