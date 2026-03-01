module tests.test_705;

fn test_705() {
    let name = "Lexicon_705"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_705!")
}


pub fn main() {
    test_705()
}
