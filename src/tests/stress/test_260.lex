module tests.test_260;

fn test_260() {
    let name = "Lexicon_260"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_260!")
}


pub fn main() {
    test_260()
}
