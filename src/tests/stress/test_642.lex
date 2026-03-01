module tests.test_642;

fn test_642() {
    let name = "Lexicon_642"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_642!")
}


pub fn main() {
    test_642()
}
