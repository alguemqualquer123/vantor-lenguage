module tests.test_235;

fn test_235() {
    let name = "Lexicon_235"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_235!")
}


pub fn main() {
    test_235()
}
