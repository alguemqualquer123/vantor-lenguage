module tests.test_75;

fn test_75() {
    let name = "Lexicon_75"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_75!")
}


pub fn main() {
    test_75()
}
