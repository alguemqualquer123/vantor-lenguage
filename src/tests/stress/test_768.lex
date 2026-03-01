module tests.test_768;

fn test_768() {
    let name = "Lexicon_768"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_768!")
}


pub fn main() {
    test_768()
}
