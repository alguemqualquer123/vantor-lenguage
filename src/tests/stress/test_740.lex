module tests.test_740;

fn test_740() {
    let val: String? = null
    let result = val ?? "default_740"
    assert(result == "default_740")
}


pub fn main() {
    test_740()
}
