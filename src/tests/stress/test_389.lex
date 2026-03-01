module tests.test_389;

fn test_389() {
    let name = "Lexicon_389"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_389!")
}


pub fn main() {
    test_389()
}
