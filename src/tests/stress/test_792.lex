module tests.test_792;

fn test_792() {
    let name = "Lexicon_792"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_792!")
}


pub fn main() {
    test_792()
}
