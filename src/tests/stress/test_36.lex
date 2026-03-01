module tests.test_36;

fn test_36() {
    let name = "Lexicon_36"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_36!")
}


pub fn main() {
    test_36()
}
