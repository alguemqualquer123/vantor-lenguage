module tests.test_618;

fn test_618() {
    let val: String? = null
    let result = val ?? "default_618"
    assert(result == "default_618")
}


pub fn main() {
    test_618()
}
