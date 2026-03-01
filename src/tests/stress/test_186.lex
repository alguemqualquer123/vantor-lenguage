module tests.test_186;

fn test_186() {
    let name = "Lexicon_186"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_186!")
}


pub fn main() {
    test_186()
}
