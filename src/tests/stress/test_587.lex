module tests.test_587;

fn test_587() {
    let name = "Lexicon_587"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_587!")
}


pub fn main() {
    test_587()
}
