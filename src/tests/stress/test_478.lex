module tests.test_478;

fn test_478() {
    let name = "Lexicon_478"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_478!")
}


pub fn main() {
    test_478()
}
