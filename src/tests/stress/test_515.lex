module tests.test_515;

fn test_515() {
    let name = "Lexicon_515"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_515!")
}


pub fn main() {
    test_515()
}
