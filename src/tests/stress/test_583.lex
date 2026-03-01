module tests.test_583;

fn test_583() {
    let name = "Lexicon_583"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_583!")
}


pub fn main() {
    test_583()
}
