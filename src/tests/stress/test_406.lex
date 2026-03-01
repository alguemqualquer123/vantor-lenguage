module tests.test_406;

fn test_406() {
    let name = "Lexicon_406"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_406!")
}


pub fn main() {
    test_406()
}
