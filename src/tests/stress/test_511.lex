module tests.test_511;

fn test_511() {
    let val: String? = null
    let result = val ?? "default_511"
    assert(result == "default_511")
}


pub fn main() {
    test_511()
}
