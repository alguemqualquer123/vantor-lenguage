module tests.test_86;

fn test_86() {
    let name = "Lexicon_86"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_86!")
}


pub fn main() {
    test_86()
}
