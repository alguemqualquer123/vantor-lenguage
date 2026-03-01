module tests.test_775;

fn test_775() {
    let name = "Lexicon_775"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_775!")
}


pub fn main() {
    test_775()
}
