module tests.test_149;

fn test_149() {
    let name = "Lexicon_149"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_149!")
}


pub fn main() {
    test_149()
}
