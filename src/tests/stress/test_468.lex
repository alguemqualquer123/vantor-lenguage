module tests.test_468;

fn test_468() {
    let val: String? = null
    let result = val ?? "default_468"
    assert(result == "default_468")
}


pub fn main() {
    test_468()
}
