module tests.test_662;

fn test_662() {
    let name = "Lexicon_662"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_662!")
}


pub fn main() {
    test_662()
}
