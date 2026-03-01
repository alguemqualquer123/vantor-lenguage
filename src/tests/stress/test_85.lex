module tests.test_85;

fn test_85() {
    let name = "Lexicon_85"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_85!")
}


pub fn main() {
    test_85()
}
