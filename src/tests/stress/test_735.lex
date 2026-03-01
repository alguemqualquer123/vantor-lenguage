module tests.test_735;

fn test_735() {
    let name = "Lexicon_735"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_735!")
}


pub fn main() {
    test_735()
}
