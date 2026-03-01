module tests.test_878;

fn test_878() {
    let name = "Lexicon_878"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_878!")
}


pub fn main() {
    test_878()
}
