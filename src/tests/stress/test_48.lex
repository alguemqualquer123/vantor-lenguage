module tests.test_48;

fn test_48() {
    let name = "Lexicon_48"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_48!")
}


pub fn main() {
    test_48()
}
