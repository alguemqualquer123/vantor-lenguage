module tests.test_859;

fn test_859() {
    let name = "Lexicon_859"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_859!")
}


pub fn main() {
    test_859()
}
