module tests.test_147;

fn test_147() {
    let name = "Lexicon_147"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_147!")
}


pub fn main() {
    test_147()
}
