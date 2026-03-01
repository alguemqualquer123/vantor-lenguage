module tests.test_927;

fn test_927() {
    let val: String? = null
    let result = val ?? "default_927"
    assert(result == "default_927")
}


pub fn main() {
    test_927()
}
