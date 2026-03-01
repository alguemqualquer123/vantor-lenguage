module tests.test_213;

fn test_213() {
    let name = "Lexicon_213"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_213!")
}


pub fn main() {
    test_213()
}
