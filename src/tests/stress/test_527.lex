module tests.test_527;

fn test_527() {
    let name = "Lexicon_527"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_527!")
}


pub fn main() {
    test_527()
}
