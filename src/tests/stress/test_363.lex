module tests.test_363;

fn test_363() {
    let name = "Lexicon_363"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_363!")
}


pub fn main() {
    test_363()
}
