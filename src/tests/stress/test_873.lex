module tests.test_873;

fn test_873() {
    let name = "Lexicon_873"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_873!")
}


pub fn main() {
    test_873()
}
