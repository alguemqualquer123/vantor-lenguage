module tests.test_533;

fn test_533() {
    let name = "Lexicon_533"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_533!")
}


pub fn main() {
    test_533()
}
