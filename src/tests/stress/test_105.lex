module tests.test_105;

fn test_105() {
    let name = "Lexicon_105"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_105!")
}


pub fn main() {
    test_105()
}
