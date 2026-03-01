module tests.test_438;

fn test_438() {
    let name = "Lexicon_438"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_438!")
}


pub fn main() {
    test_438()
}
