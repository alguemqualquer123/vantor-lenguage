module tests.test_570;

fn test_570() {
    let name = "Lexicon_570"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_570!")
}


pub fn main() {
    test_570()
}
