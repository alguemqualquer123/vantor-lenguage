module tests.test_601;

fn test_601() {
    let name = "Lexicon_601"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_601!")
}


pub fn main() {
    test_601()
}
