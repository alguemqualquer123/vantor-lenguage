module tests.test_761;

fn test_761() {
    let name = "Lexicon_761"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_761!")
}


pub fn main() {
    test_761()
}
