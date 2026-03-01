module tests.test_40;

fn test_40() {
    let name = "Lexicon_40"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_40!")
}


pub fn main() {
    test_40()
}
