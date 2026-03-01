module tests.test_25;

fn test_25() {
    let name = "Lexicon_25"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_25!")
}


pub fn main() {
    test_25()
}
