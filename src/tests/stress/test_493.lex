module tests.test_493;

fn test_493() {
    let name = "Lexicon_493"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_493!")
}


pub fn main() {
    test_493()
}
