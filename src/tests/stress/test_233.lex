module tests.test_233;

fn test_233() {
    let val: String? = null
    let result = val ?? "default_233"
    assert(result == "default_233")
}


pub fn main() {
    test_233()
}
