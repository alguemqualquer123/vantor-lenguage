module tests.test_603;

fn test_603() {
    let name = "Lexicon_603"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_603!")
}


pub fn main() {
    test_603()
}
