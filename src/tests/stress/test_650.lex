module tests.test_650;

fn test_650() {
    let name = "Lexicon_650"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_650!")
}


pub fn main() {
    test_650()
}
