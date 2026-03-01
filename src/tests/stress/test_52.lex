module tests.test_52;

fn test_52() {
    let name = "Lexicon_52"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_52!")
}


pub fn main() {
    test_52()
}
