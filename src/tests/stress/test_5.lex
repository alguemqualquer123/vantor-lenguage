module tests.test_5;

fn test_5() {
    let name = "Lexicon_5"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_5!")
}


pub fn main() {
    test_5()
}
