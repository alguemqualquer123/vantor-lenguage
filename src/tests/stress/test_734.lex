module tests.test_734;

fn test_734() {
    let name = "Lexicon_734"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_734!")
}


pub fn main() {
    test_734()
}
