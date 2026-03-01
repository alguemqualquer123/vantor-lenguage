module tests.test_338;

fn test_338() {
    let name = "Lexicon_338"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_338!")
}


pub fn main() {
    test_338()
}
