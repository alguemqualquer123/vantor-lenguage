module tests.test_884;

fn test_884() {
    let name = "Lexicon_884"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_884!")
}


pub fn main() {
    test_884()
}
