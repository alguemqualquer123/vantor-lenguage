module tests.test_913;

fn test_913() {
    let val: String? = null
    let result = val ?? "default_913"
    assert(result == "default_913")
}


pub fn main() {
    test_913()
}
