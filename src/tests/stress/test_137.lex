module tests.test_137;

fn test_137() {
    let name = "Lexicon_137"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_137!")
}


pub fn main() {
    test_137()
}
