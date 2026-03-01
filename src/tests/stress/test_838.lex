module tests.test_838;

fn test_838() {
    let name = "Lexicon_838"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_838!")
}


pub fn main() {
    test_838()
}
