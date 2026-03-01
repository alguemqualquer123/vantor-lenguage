module tests.test_759;

fn test_759() {
    let name = "Lexicon_759"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_759!")
}


pub fn main() {
    test_759()
}
