module tests.test_481;

fn test_481() {
    let val: String? = null
    let result = val ?? "default_481"
    assert(result == "default_481")
}


pub fn main() {
    test_481()
}
