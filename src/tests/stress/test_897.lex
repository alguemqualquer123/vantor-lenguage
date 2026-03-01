module tests.test_897;

fn test_897() {
    let name = "Lexicon_897"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_897!")
}


pub fn main() {
    test_897()
}
