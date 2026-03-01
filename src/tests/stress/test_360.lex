module tests.test_360;

fn test_360() {
    let name = "Lexicon_360"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_360!")
}


pub fn main() {
    test_360()
}
