module tests.test_725;

fn test_725() {
    let val: String? = null
    let result = val ?? "default_725"
    assert(result == "default_725")
}


pub fn main() {
    test_725()
}
