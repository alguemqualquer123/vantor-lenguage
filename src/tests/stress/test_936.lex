module tests.test_936;

fn test_936() {
    let name = "Lexicon_936"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_936!")
}


pub fn main() {
    test_936()
}
