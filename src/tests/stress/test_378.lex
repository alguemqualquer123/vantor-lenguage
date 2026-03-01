module tests.test_378;

fn test_378() {
    let name = "Lexicon_378"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_378!")
}


pub fn main() {
    test_378()
}
