module tests.test_195;

fn test_195() {
    let val: String? = null
    let result = val ?? "default_195"
    assert(result == "default_195")
}


pub fn main() {
    test_195()
}
