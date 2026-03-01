module tests.test_92;

fn test_92() {
    let name = "Lexicon_92"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_92!")
}


pub fn main() {
    test_92()
}
