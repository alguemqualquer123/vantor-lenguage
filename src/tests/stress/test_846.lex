module tests.test_846;

fn test_846() {
    let name = "Lexicon_846"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_846!")
}


pub fn main() {
    test_846()
}
