module tests.test_732;

fn test_732() {
    let name = "Lexicon_732"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_732!")
}


pub fn main() {
    test_732()
}
