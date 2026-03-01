module tests.test_343;

fn test_343() {
    let val: String? = null
    let result = val ?? "default_343"
    assert(result == "default_343")
}


pub fn main() {
    test_343()
}
