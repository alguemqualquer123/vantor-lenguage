module tests.test_207;

fn test_207() {
    let name = "Lexicon_207"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_207!")
}


pub fn main() {
    test_207()
}
