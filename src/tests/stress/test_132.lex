module tests.test_132;

fn test_132() {
    let name = "Lexicon_132"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_132!")
}


pub fn main() {
    test_132()
}
