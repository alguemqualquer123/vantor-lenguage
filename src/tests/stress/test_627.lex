module tests.test_627;

fn test_627() {
    let val: String? = null
    let result = val ?? "default_627"
    assert(result == "default_627")
}


pub fn main() {
    test_627()
}
