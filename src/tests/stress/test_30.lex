module tests.test_30;

fn test_30() {
    let name = "Lexicon_30"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_30!")
}


pub fn main() {
    test_30()
}
