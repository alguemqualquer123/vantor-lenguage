module tests.test_415;

fn test_415() {
    let name = "Lexicon_415"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_415!")
}


pub fn main() {
    test_415()
}
