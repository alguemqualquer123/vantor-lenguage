module tests.test_176;

fn test_176() {
    let val: String? = null
    let result = val ?? "default_176"
    assert(result == "default_176")
}


pub fn main() {
    test_176()
}
