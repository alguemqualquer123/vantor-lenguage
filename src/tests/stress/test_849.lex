module tests.test_849;

fn test_849() {
    let name = "Lexicon_849"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_849!")
}


pub fn main() {
    test_849()
}
