module tests.test_801;

fn test_801() {
    let name = "Lexicon_801"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_801!")
}


pub fn main() {
    test_801()
}
