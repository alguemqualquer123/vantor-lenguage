module tests.test_685;

fn test_685() {
    let name = "Lexicon_685"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_685!")
}


pub fn main() {
    test_685()
}
