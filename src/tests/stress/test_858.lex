module tests.test_858;

fn test_858() {
    let val: String? = null
    let result = val ?? "default_858"
    assert(result == "default_858")
}


pub fn main() {
    test_858()
}
