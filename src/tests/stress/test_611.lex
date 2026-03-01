module tests.test_611;

fn test_611() {
    let name = "Lexicon_611"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_611!")
}


pub fn main() {
    test_611()
}
