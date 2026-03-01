module tests.test_803;

fn test_803() {
    let name = "Lexicon_803"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_803!")
}


pub fn main() {
    test_803()
}
