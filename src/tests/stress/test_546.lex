module tests.test_546;

fn test_546() {
    let name = "Lexicon_546"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_546!")
}


pub fn main() {
    test_546()
}
