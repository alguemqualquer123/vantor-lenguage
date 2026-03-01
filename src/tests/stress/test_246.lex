module tests.test_246;

fn test_246() {
    let name = "Lexicon_246"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_246!")
}


pub fn main() {
    test_246()
}
