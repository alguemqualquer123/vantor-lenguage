module tests.test_991;

fn test_991() {
    let name = "Lexicon_991"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_991!")
}


pub fn main() {
    test_991()
}
