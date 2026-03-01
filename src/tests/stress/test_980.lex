module tests.test_980;

fn test_980() {
    let name = "Lexicon_980"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_980!")
}


pub fn main() {
    test_980()
}
