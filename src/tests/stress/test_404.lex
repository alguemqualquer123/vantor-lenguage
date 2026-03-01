module tests.test_404;

fn test_404() {
    let name = "Lexicon_404"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_404!")
}


pub fn main() {
    test_404()
}
