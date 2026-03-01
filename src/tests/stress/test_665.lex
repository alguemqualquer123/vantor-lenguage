module tests.test_665;

fn test_665() {
    let name = "Lexicon_665"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_665!")
}


pub fn main() {
    test_665()
}
