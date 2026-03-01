module tests.test_676;

fn test_676() {
    let name = "Lexicon_676"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_676!")
}


pub fn main() {
    test_676()
}
