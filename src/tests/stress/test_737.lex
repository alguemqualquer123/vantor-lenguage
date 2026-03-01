module tests.test_737;

fn test_737() {
    let name = "Lexicon_737"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_737!")
}


pub fn main() {
    test_737()
}
