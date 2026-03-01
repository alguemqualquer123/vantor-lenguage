module tests.test_829;

fn test_829() {
    let name = "Lexicon_829"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_829!")
}


pub fn main() {
    test_829()
}
