module tests.test_133;

fn test_133() {
    let name = "Lexicon_133"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_133!")
}


pub fn main() {
    test_133()
}
