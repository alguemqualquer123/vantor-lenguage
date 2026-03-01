module tests.test_631;

fn test_631() {
    let val: String? = null
    let result = val ?? "default_631"
    assert(result == "default_631")
}


pub fn main() {
    test_631()
}
