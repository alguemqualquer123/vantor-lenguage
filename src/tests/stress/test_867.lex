module tests.test_867;

fn test_867() {
    let name = "Lexicon_867"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_867!")
}


pub fn main() {
    test_867()
}
