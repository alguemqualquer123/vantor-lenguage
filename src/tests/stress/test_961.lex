module tests.test_961;

fn test_961() {
    let name = "Lexicon_961"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_961!")
}


pub fn main() {
    test_961()
}
