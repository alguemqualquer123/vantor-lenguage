module tests.test_117;

fn test_117() {
    let name = "Lexicon_117"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_117!")
}


pub fn main() {
    test_117()
}
