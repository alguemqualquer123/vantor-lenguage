module tests.test_262;

fn test_262() {
    let name = "Lexicon_262"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_262!")
}


pub fn main() {
    test_262()
}
