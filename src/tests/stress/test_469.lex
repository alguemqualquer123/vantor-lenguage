module tests.test_469;

fn test_469() {
    let val: String? = null
    let result = val ?? "default_469"
    assert(result == "default_469")
}


pub fn main() {
    test_469()
}
