module tests.test_334;

fn test_334() {
    let name = "Lexicon_334"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_334!")
}


pub fn main() {
    test_334()
}
