module tests.test_330;

fn test_330() {
    let name = "Lexicon_330"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_330!")
}


pub fn main() {
    test_330()
}
