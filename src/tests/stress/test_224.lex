module tests.test_224;

fn test_224() {
    let name = "Lexicon_224"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_224!")
}


pub fn main() {
    test_224()
}
