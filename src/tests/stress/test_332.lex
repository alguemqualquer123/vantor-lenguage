module tests.test_332;

fn test_332() {
    let name = "Lexicon_332"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_332!")
}


pub fn main() {
    test_332()
}
