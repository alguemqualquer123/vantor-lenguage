module tests.test_767;

fn test_767() {
    let val: String? = null
    let result = val ?? "default_767"
    assert(result == "default_767")
}


pub fn main() {
    test_767()
}
