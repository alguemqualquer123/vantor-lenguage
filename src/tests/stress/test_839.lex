module tests.test_839;

fn test_839() {
    let name = "Lexicon_839"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_839!")
}


pub fn main() {
    test_839()
}
