module tests.test_742;

fn test_742() {
    let name = "Lexicon_742"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_742!")
}


pub fn main() {
    test_742()
}
