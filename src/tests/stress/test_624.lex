module tests.test_624;

fn test_624() {
    let name = "Lexicon_624"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_624!")
}


pub fn main() {
    test_624()
}
