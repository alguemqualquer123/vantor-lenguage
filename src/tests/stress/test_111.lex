module tests.test_111;

fn test_111() {
    let name = "Lexicon_111"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_111!")
}


pub fn main() {
    test_111()
}
