module tests.test_311;

fn test_311() {
    let name = "Lexicon_311"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_311!")
}


pub fn main() {
    test_311()
}
