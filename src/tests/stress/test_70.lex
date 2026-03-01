module tests.test_70;

fn test_70() {
    let val: String? = null
    let result = val ?? "default_70"
    assert(result == "default_70")
}


pub fn main() {
    test_70()
}
