module tests.test_567;

fn test_567() {
    let name = "Lexicon_567"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_567!")
}


pub fn main() {
    test_567()
}
