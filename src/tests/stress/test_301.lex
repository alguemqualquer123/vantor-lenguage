module tests.test_301;

fn test_301() {
    let val: String? = null
    let result = val ?? "default_301"
    assert(result == "default_301")
}


pub fn main() {
    test_301()
}
