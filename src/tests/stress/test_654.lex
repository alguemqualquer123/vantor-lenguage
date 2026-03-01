module tests.test_654;

fn test_654() {
    let val: String? = null
    let result = val ?? "default_654"
    assert(result == "default_654")
}


pub fn main() {
    test_654()
}
