module tests.test_517;

fn test_517() {
    let val: String? = null
    let result = val ?? "default_517"
    assert(result == "default_517")
}


pub fn main() {
    test_517()
}
