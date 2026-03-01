module tests.test_881;

fn test_881() {
    let name = "Lexicon_881"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_881!")
}


pub fn main() {
    test_881()
}
