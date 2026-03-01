module tests.test_636;

fn test_636() {
    let name = "Lexicon_636"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_636!")
}


pub fn main() {
    test_636()
}
