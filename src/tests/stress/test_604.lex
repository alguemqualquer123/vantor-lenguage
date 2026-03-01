module tests.test_604;

fn test_604() {
    let name = "Lexicon_604"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_604!")
}


pub fn main() {
    test_604()
}
