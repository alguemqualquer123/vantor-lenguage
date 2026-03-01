module tests.test_495;

fn test_495() {
    let name = "Lexicon_495"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_495!")
}


pub fn main() {
    test_495()
}
