module tests.test_440;

fn test_440() {
    let name = "Lexicon_440"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_440!")
}


pub fn main() {
    test_440()
}
