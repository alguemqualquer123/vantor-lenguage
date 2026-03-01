module tests.test_816;

fn test_816() {
    let val: String? = null
    let result = val ?? "default_816"
    assert(result == "default_816")
}


pub fn main() {
    test_816()
}
