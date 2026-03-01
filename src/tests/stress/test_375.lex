module tests.test_375;

fn test_375() {
    let name = "Lexicon_375"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_375!")
}


pub fn main() {
    test_375()
}
