module tests.test_862;

fn test_862() {
    let name = "Lexicon_862"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_862!")
}


pub fn main() {
    test_862()
}
