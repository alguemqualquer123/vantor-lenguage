module tests.test_891;

fn test_891() {
    let name = "Lexicon_891"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_891!")
}


pub fn main() {
    test_891()
}
