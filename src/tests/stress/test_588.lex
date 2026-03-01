module tests.test_588;

fn test_588() {
    let name = "Lexicon_588"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_588!")
}


pub fn main() {
    test_588()
}
