module tests.test_629;

fn test_629() {
    let val: String? = null
    let result = val ?? "default_629"
    assert(result == "default_629")
}


pub fn main() {
    test_629()
}
