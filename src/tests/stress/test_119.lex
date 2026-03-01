module tests.test_119;

fn test_119() {
    let val: String? = null
    let result = val ?? "default_119"
    assert(result == "default_119")
}


pub fn main() {
    test_119()
}
