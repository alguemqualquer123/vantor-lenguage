module tests.test_753;

fn test_753() {
    let name = "Lexicon_753"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_753!")
}


pub fn main() {
    test_753()
}
