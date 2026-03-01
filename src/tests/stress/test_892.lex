module tests.test_892;

fn test_892() {
    let name = "Lexicon_892"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_892!")
}


pub fn main() {
    test_892()
}
