module tests.test_37;

fn test_37() {
    let name = "Lexicon_37"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_37!")
}


pub fn main() {
    test_37()
}
