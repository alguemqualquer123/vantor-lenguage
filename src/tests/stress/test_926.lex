module tests.test_926;

fn test_926() {
    let name = "Lexicon_926"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_926!")
}


pub fn main() {
    test_926()
}
