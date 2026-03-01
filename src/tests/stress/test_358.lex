module tests.test_358;

fn test_358() {
    let val: String? = null
    let result = val ?? "default_358"
    assert(result == "default_358")
}


pub fn main() {
    test_358()
}
