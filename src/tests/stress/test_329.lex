module tests.test_329;

fn test_329() {
    let name = "Lexicon_329"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_329!")
}


pub fn main() {
    test_329()
}
