module tests.test_15;

fn test_15() {
    let name = "Lexicon_15"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_15!")
}


pub fn main() {
    test_15()
}
