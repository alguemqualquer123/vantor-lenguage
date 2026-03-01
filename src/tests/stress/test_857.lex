module tests.test_857;

fn test_857() {
    let name = "Lexicon_857"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_857!")
}


pub fn main() {
    test_857()
}
