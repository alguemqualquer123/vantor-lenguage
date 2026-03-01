module tests.test_333;

fn test_333() {
    let name = "Lexicon_333"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_333!")
}


pub fn main() {
    test_333()
}
