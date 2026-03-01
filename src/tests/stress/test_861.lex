module tests.test_861;

fn test_861() {
    let name = "Lexicon_861"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_861!")
}


pub fn main() {
    test_861()
}
