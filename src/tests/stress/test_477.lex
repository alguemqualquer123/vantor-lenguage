module tests.test_477;

fn test_477() {
    let val: String? = null
    let result = val ?? "default_477"
    assert(result == "default_477")
}


pub fn main() {
    test_477()
}
