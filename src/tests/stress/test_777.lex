module tests.test_777;

fn test_777() {
    let name = "Lexicon_777"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_777!")
}


pub fn main() {
    test_777()
}
