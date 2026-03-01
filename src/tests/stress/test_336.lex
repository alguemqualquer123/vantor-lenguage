module tests.test_336;

fn test_336() {
    let name = "Lexicon_336"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_336!")
}


pub fn main() {
    test_336()
}
