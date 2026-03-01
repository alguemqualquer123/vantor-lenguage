module tests.test_502;

fn test_502() {
    let name = "Lexicon_502"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_502!")
}


pub fn main() {
    test_502()
}
