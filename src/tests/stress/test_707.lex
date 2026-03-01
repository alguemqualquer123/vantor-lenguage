module tests.test_707;

fn test_707() {
    let name = "Lexicon_707"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_707!")
}


pub fn main() {
    test_707()
}
