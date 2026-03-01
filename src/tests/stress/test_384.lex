module tests.test_384;

fn test_384() {
    let name = "Lexicon_384"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_384!")
}


pub fn main() {
    test_384()
}
