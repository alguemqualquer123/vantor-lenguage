module tests.test_763;

fn test_763() {
    let val: String? = null
    let result = val ?? "default_763"
    assert(result == "default_763")
}


pub fn main() {
    test_763()
}
