module tests.test_288;

fn test_288() {
    let name = "Lexicon_288"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_288!")
}


pub fn main() {
    test_288()
}
