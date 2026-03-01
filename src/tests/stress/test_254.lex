module tests.test_254;

fn test_254() {
    let name = "Lexicon_254"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_254!")
}


pub fn main() {
    test_254()
}
