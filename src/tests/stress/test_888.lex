module tests.test_888;

fn test_888() {
    let name = "Lexicon_888"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_888!")
}


pub fn main() {
    test_888()
}
