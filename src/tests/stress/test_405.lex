module tests.test_405;

fn test_405() {
    let name = "Lexicon_405"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_405!")
}


pub fn main() {
    test_405()
}
