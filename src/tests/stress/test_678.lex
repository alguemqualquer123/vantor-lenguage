module tests.test_678;

fn test_678() {
    let name = "Lexicon_678"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_678!")
}


pub fn main() {
    test_678()
}
