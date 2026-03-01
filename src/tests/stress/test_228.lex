module tests.test_228;

fn test_228() {
    let name = "Lexicon_228"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_228!")
}


pub fn main() {
    test_228()
}
