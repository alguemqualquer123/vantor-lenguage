module tests.test_595;

fn test_595() {
    let name = "Lexicon_595"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_595!")
}


pub fn main() {
    test_595()
}
