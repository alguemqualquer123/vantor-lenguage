module tests.test_851;

fn test_851() {
    let name = "Lexicon_851"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_851!")
}


pub fn main() {
    test_851()
}
