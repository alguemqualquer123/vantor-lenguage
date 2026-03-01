module tests.test_610;

fn test_610() {
    let val: String? = null
    let result = val ?? "default_610"
    assert(result == "default_610")
}


pub fn main() {
    test_610()
}
