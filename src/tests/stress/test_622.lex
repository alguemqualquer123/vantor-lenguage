module tests.test_622;

fn test_622() {
    let name = "Lexicon_622"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_622!")
}


pub fn main() {
    test_622()
}
