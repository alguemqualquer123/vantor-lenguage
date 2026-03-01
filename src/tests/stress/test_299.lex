module tests.test_299;

fn test_299() {
    let name = "Lexicon_299"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_299!")
}


pub fn main() {
    test_299()
}
