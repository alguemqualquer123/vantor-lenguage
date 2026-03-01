module tests.test_76;

fn test_76() {
    let name = "Lexicon_76"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_76!")
}


pub fn main() {
    test_76()
}
