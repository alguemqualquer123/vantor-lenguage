module tests.test_81;

fn test_81() {
    let val: String? = null
    let result = val ?? "default_81"
    assert(result == "default_81")
}


pub fn main() {
    test_81()
}
