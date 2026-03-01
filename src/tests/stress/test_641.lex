module tests.test_641;

fn test_641() {
    let name = "Lexicon_641"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_641!")
}


pub fn main() {
    test_641()
}
