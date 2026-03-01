module tests.test_241;

fn test_241() {
    let name = "Lexicon_241"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_241!")
}


pub fn main() {
    test_241()
}
