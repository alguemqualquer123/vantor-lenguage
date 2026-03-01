module tests.test_720;

fn test_720() {
    let name = "Lexicon_720"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_720!")
}


pub fn main() {
    test_720()
}
