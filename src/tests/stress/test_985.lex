module tests.test_985;

fn test_985() {
    let name = "Lexicon_985"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_985!")
}


pub fn main() {
    test_985()
}
