module tests.test_297;

fn test_297() {
    let name = "Lexicon_297"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_297!")
}


pub fn main() {
    test_297()
}
