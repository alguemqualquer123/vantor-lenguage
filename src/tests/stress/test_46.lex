module tests.test_46;

fn test_46() {
    let name = "Lexicon_46"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_46!")
}


pub fn main() {
    test_46()
}
