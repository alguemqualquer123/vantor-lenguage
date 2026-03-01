module tests.test_168;

fn test_168() {
    let name = "Lexicon_168"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_168!")
}


pub fn main() {
    test_168()
}
