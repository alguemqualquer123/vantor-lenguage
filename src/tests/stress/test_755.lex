module tests.test_755;

fn test_755() {
    let name = "Lexicon_755"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_755!")
}


pub fn main() {
    test_755()
}
