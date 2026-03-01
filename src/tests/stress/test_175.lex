module tests.test_175;

fn test_175() {
    let name = "Lexicon_175"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_175!")
}


pub fn main() {
    test_175()
}
