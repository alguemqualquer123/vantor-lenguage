module tests.test_706;

fn test_706() {
    let name = "Lexicon_706"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_706!")
}


pub fn main() {
    test_706()
}
