module tests.test_475;

fn test_475() {
    let name = "Lexicon_475"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_475!")
}


pub fn main() {
    test_475()
}
