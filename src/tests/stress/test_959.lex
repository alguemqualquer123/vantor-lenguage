module tests.test_959;

fn test_959() {
    let val: String? = null
    let result = val ?? "default_959"
    assert(result == "default_959")
}


pub fn main() {
    test_959()
}
