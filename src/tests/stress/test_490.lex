module tests.test_490;

fn test_490() {
    let name = "Lexicon_490"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_490!")
}


pub fn main() {
    test_490()
}
