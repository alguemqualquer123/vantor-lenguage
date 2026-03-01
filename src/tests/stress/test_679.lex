module tests.test_679;

fn test_679() {
    let val: String? = null
    let result = val ?? "default_679"
    assert(result == "default_679")
}


pub fn main() {
    test_679()
}
