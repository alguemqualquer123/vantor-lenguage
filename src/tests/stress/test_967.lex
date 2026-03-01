module tests.test_967;

fn test_967() {
    let val: String? = null
    let result = val ?? "default_967"
    assert(result == "default_967")
}


pub fn main() {
    test_967()
}
