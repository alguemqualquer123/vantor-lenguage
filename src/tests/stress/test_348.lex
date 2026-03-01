module tests.test_348;

fn test_348() {
    let name = "Lexicon_348"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_348!")
}


pub fn main() {
    test_348()
}
