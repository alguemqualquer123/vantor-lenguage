module tests.test_652;

fn test_652() {
    let name = "Lexicon_652"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_652!")
}


pub fn main() {
    test_652()
}
