module tests.test_594;

fn test_594() {
    let name = "Lexicon_594"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_594!")
}


pub fn main() {
    test_594()
}
