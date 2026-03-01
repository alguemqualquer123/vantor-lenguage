module tests.test_421;

fn test_421() {
    let name = "Lexicon_421"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_421!")
}


pub fn main() {
    test_421()
}
