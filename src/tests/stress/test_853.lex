module tests.test_853;

fn test_853() {
    let name = "Lexicon_853"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_853!")
}


pub fn main() {
    test_853()
}
