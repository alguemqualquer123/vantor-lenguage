module tests.test_984;

fn test_984() {
    let name = "Lexicon_984"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_984!")
}


pub fn main() {
    test_984()
}
