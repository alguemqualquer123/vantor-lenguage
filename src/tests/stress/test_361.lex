module tests.test_361;

fn test_361() {
    let name = "Lexicon_361"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_361!")
}


pub fn main() {
    test_361()
}
