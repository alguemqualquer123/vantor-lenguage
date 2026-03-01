module tests.test_103;

fn test_103() {
    let name = "Lexicon_103"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_103!")
}


pub fn main() {
    test_103()
}
