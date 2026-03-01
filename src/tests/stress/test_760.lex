module tests.test_760;

fn test_760() {
    let name = "Lexicon_760"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_760!")
}


pub fn main() {
    test_760()
}
