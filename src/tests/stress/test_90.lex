module tests.test_90;

fn test_90() {
    let name = "Lexicon_90"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_90!")
}


pub fn main() {
    test_90()
}
