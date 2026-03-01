module tests.test_350;

fn test_350() {
    let name = "Lexicon_350"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_350!")
}


pub fn main() {
    test_350()
}
