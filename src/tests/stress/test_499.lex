module tests.test_499;

fn test_499() {
    let val: String? = null
    let result = val ?? "default_499"
    assert(result == "default_499")
}


pub fn main() {
    test_499()
}
