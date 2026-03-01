module tests.test_467;

fn test_467() {
    let name = "Lexicon_467"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_467!")
}


pub fn main() {
    test_467()
}
