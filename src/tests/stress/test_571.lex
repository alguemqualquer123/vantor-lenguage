module tests.test_571;

fn test_571() {
    let val: String? = null
    let result = val ?? "default_571"
    assert(result == "default_571")
}


pub fn main() {
    test_571()
}
