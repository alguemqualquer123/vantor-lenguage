module tests.test_178;

fn test_178() {
    let name = "Lexicon_178"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_178!")
}


pub fn main() {
    test_178()
}
