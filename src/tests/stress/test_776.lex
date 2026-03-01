module tests.test_776;

fn test_776() {
    let name = "Lexicon_776"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_776!")
}


pub fn main() {
    test_776()
}
