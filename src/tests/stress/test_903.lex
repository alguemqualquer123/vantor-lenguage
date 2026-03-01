module tests.test_903;

fn test_903() {
    let name = "Lexicon_903"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_903!")
}


pub fn main() {
    test_903()
}
