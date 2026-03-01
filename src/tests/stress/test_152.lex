module tests.test_152;

fn test_152() {
    let name = "Lexicon_152"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_152!")
}


pub fn main() {
    test_152()
}
