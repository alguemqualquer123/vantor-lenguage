module tests.test_38;

fn test_38() {
    let name = "Lexicon_38"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_38!")
}


pub fn main() {
    test_38()
}
