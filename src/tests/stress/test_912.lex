module tests.test_912;

fn test_912() {
    let name = "Lexicon_912"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_912!")
}


pub fn main() {
    test_912()
}
