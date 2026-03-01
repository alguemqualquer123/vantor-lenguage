module tests.test_713;

fn test_713() {
    let name = "Lexicon_713"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_713!")
}


pub fn main() {
    test_713()
}
