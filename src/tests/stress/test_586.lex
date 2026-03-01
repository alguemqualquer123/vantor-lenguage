module tests.test_586;

fn test_586() {
    let val: String? = null
    let result = val ?? "default_586"
    assert(result == "default_586")
}


pub fn main() {
    test_586()
}
