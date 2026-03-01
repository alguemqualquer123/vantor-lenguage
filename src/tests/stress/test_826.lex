module tests.test_826;

fn test_826() {
    let name = "Lexicon_826"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_826!")
}


pub fn main() {
    test_826()
}
