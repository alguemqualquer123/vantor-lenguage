module tests.test_812;

fn test_812() {
    let val: String? = null
    let result = val ?? "default_812"
    assert(result == "default_812")
}


pub fn main() {
    test_812()
}
