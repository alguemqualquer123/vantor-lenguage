module tests.test_273;

fn test_273() {
    let val: String? = null
    let result = val ?? "default_273"
    assert(result == "default_273")
}


pub fn main() {
    test_273()
}
