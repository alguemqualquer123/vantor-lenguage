module tests.test_143;

fn test_143() {
    let name = "Lexicon_143"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_143!")
}


pub fn main() {
    test_143()
}
