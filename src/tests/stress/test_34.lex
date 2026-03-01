module tests.test_34;

fn test_34() {
    let name = "Lexicon_34"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_34!")
}


pub fn main() {
    test_34()
}
