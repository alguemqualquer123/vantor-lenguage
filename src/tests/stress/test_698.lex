module tests.test_698;

fn test_698() {
    let name = "Lexicon_698"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_698!")
}


pub fn main() {
    test_698()
}
