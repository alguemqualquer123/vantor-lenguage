module tests.test_420;

fn test_420() {
    let name = "Lexicon_420"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_420!")
}


pub fn main() {
    test_420()
}
