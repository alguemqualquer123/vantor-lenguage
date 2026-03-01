module tests.test_958;

fn test_958() {
    let name = "Lexicon_958"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_958!")
}


pub fn main() {
    test_958()
}
