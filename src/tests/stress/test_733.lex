module tests.test_733;

fn test_733() {
    let name = "Lexicon_733"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_733!")
}


pub fn main() {
    test_733()
}
