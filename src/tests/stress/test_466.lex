module tests.test_466;

fn test_466() {
    let val: String? = null
    let result = val ?? "default_466"
    assert(result == "default_466")
}


pub fn main() {
    test_466()
}
