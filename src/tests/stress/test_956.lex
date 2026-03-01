module tests.test_956;

fn test_956() {
    let name = "Lexicon_956"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_956!")
}


pub fn main() {
    test_956()
}
