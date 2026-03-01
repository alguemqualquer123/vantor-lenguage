module tests.test_687;

fn test_687() {
    let name = "Lexicon_687"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_687!")
}


pub fn main() {
    test_687()
}
