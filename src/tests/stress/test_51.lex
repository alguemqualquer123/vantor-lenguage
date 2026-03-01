module tests.test_51;

fn test_51() {
    let name = "Lexicon_51"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_51!")
}


pub fn main() {
    test_51()
}
