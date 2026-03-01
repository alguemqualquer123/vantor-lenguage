module tests.test_815;

fn test_815() {
    let name = "Lexicon_815"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_815!")
}


pub fn main() {
    test_815()
}
