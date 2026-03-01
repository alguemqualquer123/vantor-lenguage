module tests.test_894;

fn test_894() {
    let name = "Lexicon_894"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_894!")
}


pub fn main() {
    test_894()
}
