module tests.test_324;

fn test_324() {
    let name = "Lexicon_324"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_324!")
}


pub fn main() {
    test_324()
}
