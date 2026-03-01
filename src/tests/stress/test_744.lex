module tests.test_744;

fn test_744() {
    let val: String? = null
    let result = val ?? "default_744"
    assert(result == "default_744")
}


pub fn main() {
    test_744()
}
