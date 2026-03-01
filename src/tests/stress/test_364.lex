module tests.test_364;

fn test_364() {
    let name = "Lexicon_364"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_364!")
}


pub fn main() {
    test_364()
}
