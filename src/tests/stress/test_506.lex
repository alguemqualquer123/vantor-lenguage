module tests.test_506;

fn test_506() {
    let val: String? = null
    let result = val ?? "default_506"
    assert(result == "default_506")
}


pub fn main() {
    test_506()
}
