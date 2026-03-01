module tests.test_249;

fn test_249() {
    let val: String? = null
    let result = val ?? "default_249"
    assert(result == "default_249")
}


pub fn main() {
    test_249()
}
