module tests.test_317;

fn test_317() {
    let name = "Lexicon_317"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_317!")
}


pub fn main() {
    test_317()
}
