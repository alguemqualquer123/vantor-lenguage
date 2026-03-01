module tests.test_751;

fn test_751() {
    let name = "Lexicon_751"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_751!")
}


pub fn main() {
    test_751()
}
