module tests.test_314;

fn test_314() {
    let name = "Lexicon_314"
    let msg = "Hello, {name}!"
    assert(msg == "Hello, Lexicon_314!")
}


pub fn main() {
    test_314()
}
