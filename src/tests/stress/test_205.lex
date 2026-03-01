module tests.test_205;

fn test_205() {
    let name = "Lexicon_205"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_205!")
}


pub fn main() {
    test_205()
}
