module tests.test_316;

fn test_316() {
    let name = "Lexicon_316"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_316!")
}


pub fn main() {
    test_316()
}
