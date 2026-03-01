module tests.test_640;

fn test_640() {
    let name = "Lexicon_640"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_640!")
}


pub fn main() {
    test_640()
}
